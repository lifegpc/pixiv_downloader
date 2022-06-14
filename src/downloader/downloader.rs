use super::pd_file::PdFile;
use super::pd_file::PdFileResult;
use super::enums::DownloaderResult;
use super::enums::DownloaderStatus;
use super::error::DownloaderError;
use super::local_file::LocalFile;
use super::tasks::check_tasks;
use crate::ext::atomic::AtomicQuick;
use crate::ext::io::ClearFile;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::opthelper::OptHelper;
use crate::utils::ask_need_overwrite;
use crate::utils::get_file_name_from_url;
use crate::webclient::WebClient;
use crate::webclient::ToHeaders;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use indicatif::MultiProgress;
use reqwest::IntoUrl;
use std::borrow::Cow;
use std::collections::HashMap;
#[cfg(test)]
use std::fs::create_dir;
use std::fs::remove_file;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::task::JoinHandle;
use url::Url;

/// Get the target file name
pub trait GetTargetFileName {
    /// Get the target file name.
    /// The target file name just used in the message in progress bar.
    /// 
    /// If is unknown, return [None] is fine.
    fn get_target_file_name(&self) -> Option<String> {
        None
    }
}

#[derive(Debug)]
/// A file downloader
pub struct DownloaderInternal<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName> {
    /// The webclient
    pub client: Arc<WebClient>,
    /// The download status
    pub pd: Arc<PdFile>,
    /// The url of the file
    pub url: Arc<Url>,
    /// The HTTP headers map
    pub headers: Arc<HashMap<String, String>>,
    /// The target file
    file: RwLock<Option<T>>,
    /// The status of the downloader
    status: RwLock<DownloaderStatus>,
    /// All tasks
    pub tasks: RwLock<Vec<JoinHandle<Result<(), DownloaderError>>>>,
    /// Whether to enable mulitple thread mode
    multi: AtomicBool,
    /// Whether to enable the progess bar.
    progress_bar: AtomicBool,
    /// The progress bar.
    progress: RwLock<Option<ProgressBar>>,
}

impl DownloaderInternal<LocalFile> {
    /// Create a new [DownloaderInternal] instance
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(url: U, headers: H, path: Option<&P>, overwrite: Option<bool>) -> Result<DownloaderResult<Self>, DownloaderError> {
        let h = match headers.to_headers() {
            Some(h) => { h }
            None => { HashMap::new() }
        };
        let mut already_exists = false;
        let pd_file = match path {
            Some(p) => {
                let p = p.as_ref();
                match PdFile::open(p)? {
                    PdFileResult::TargetExisted => {
                        match overwrite {
                            Some(overwrite) => {
                                if !overwrite {
                                    return Ok(DownloaderResult::Canceled);
                                } else {
                                    remove_file(p)?;
                                    PdFile::new()
                                }
                            }
                            None => {
                                if !ask_need_overwrite(p.to_str().unwrap()) {
                                    return Ok(DownloaderResult::Canceled);
                                } else {
                                    remove_file(p)?;
                                    PdFile::new()
                                }
                            }
                        }
                    }
                    PdFileResult::Ok(p) => { p }
                    PdFileResult::ExistedOk(p) => {
                        already_exists = true;
                        p
                    }
                }
            }
            None => { PdFile::new() }
        };
        let file = match path {
            Some(p) => {
                if already_exists {
                    Some(LocalFile::open(p)?)
                } else {
                    Some(LocalFile::create(p)?)
                }
            }
            None => { None }
        };
        Ok(DownloaderResult::Ok(Self {
            client: Arc::new(WebClient::default()),
            pd: Arc::new(pd_file),
            url: Arc::new(url.into_url()?),
            headers: Arc::new(h),
            file: RwLock::new(file),
            status: RwLock::new(DownloaderStatus::Created),
            tasks: RwLock::new(Vec::new()),
            multi: AtomicBool::new(false),
            progress_bar: AtomicBool::new(false),
            progress: RwLock::new(None),
        }))
    }
}

impl <T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName> DownloaderInternal<T> {
    /// Add a new task to tasks
    /// * `task` - Task
    pub fn add_task(&self, task: JoinHandle<Result<(), DownloaderError>>) {
        self.tasks.get_mut().push(task)
    }

    /// Clear all datas in file
    pub fn clear_file(&self) -> std::io::Result<()> {
        match self.file.get_mut().deref_mut() {
            Some(f) => { f.clear_file()? }
            None => {}
        };
        Ok(())
    }

    /// Disable the progress bar
    pub fn disable_progress_bar(&self) {
        self.progress_bar.qstore(false);
        self.progress.get_mut().take();
    }

    /// Enable the progress bar
    /// * `style` - The style of the progress bar
    /// * `mults` - The instance of [MultiProgress] if multiple progress bars are needed.
    pub fn enable_progress_bar(&self, style: ProgressStyle, mults: Option<&MultiProgress>) {
        let mut bar = ProgressBar::new(0).with_style(style);
        match mults {
            Some(bars) => {
                bar = bars.add(bar);
            }
            None => { }
        }
        self.progress_bar.qstore(true);
        self.progress.get_mut().replace(bar);
    }

    #[inline]
    /// Returns true if the progress bar is enabled.
    pub fn enabled_progress_bar(&self) -> bool {
        self.progress_bar.qload()
    }

    #[inline]
    /// Finishes the progress bar and sets a message
    pub fn finish_progress_bar_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        match self.progress.get_ref().deref() {
            Some(p) => { p.finish_with_message(msg) }
            None => { }
        }
    }

    #[inline]
    /// Get the file name from url.
    /// If not available, use `(Unknown)`
    pub fn get_file_name(&self) -> String {
        get_file_name_from_url(self.url.deref().clone()).unwrap_or(String::from(gettext("(Unknown)")))
    }

    #[inline]
    /// Return the status of the downloader.
    pub fn get_status(&self) -> DownloaderStatus {
        self.status.get_ref().clone()
    }

    #[inline]
    /// Get the target file name
    pub fn get_target_file_name(&self) -> Option<String> {
        match self.file.get_ref().deref() {
            Some(f) => { f.get_target_file_name() }
            None => { None }
        }
    }

    #[inline]
    /// Advances the position of the progress bar by `delta`
    pub fn inc_progress_bar(&self, delta: u64) {
        match self.progress.get_ref().deref() {
            Some(p) => { p.inc(delta) }
            None => { }
        }
    }

    #[inline]
    /// Returns true if the downloader is created just now.
    pub fn is_created(&self) -> bool {
        *self.status.get_ref() == DownloaderStatus::Created
    }

    #[inline]
    /// Returns true if the downloader is downloading now.
    pub fn is_downloading(&self) -> bool {
        *self.status.get_ref() == DownloaderStatus::Downloading
    }

    #[inline]
    /// Returns true if the downloader is downloaded complete.
    pub fn is_downloaded(&self) -> bool {
        *self.status.get_ref() == DownloaderStatus::Downloaded
    }

    #[inline]
    /// Returns true if is multiple thread mode.
    pub fn is_multi_threads(&self) -> bool {
        if self.pd.is_downloading() {
            self.pd.is_multi_threads()
        } else {
            self.multi.qload()
        }
    }

    /// Seek in the file.
    /// * `data` - Data
    pub fn seek(&self, pos: SeekFrom) -> Result<u64, DownloaderError> {
        match self.file.get_mut().deref_mut() {
            Some(f) => { Ok(f.seek(pos)?) }
            None => { Ok(0) }
        }
    }

    #[inline]
    /// Set the status to [DownloaderStatus::Downloading] and returns the current value
    pub fn set_downloading(&self) -> DownloaderStatus {
        self.status.replace_with2(DownloaderStatus::Downloading)
    }

    #[inline]
    /// Set the status to [DownloaderStatus::Downloaded] and returns the current value
    pub fn set_downloaded(&self) -> DownloaderStatus {
        self.status.replace_with2(DownloaderStatus::Downloaded)
    }

    #[inline]
    /// Sets the length of the progress bar
    pub fn set_progress_bar_length(&self, length: u64) {
        match self.progress.get_ref().deref() {
            Some(p) => { p.set_length(length) }
            None => { }
        }
    }

    #[inline]
    /// Sets the position of the progress bar
    pub fn set_progress_bar_position(&self, pos: u64) {
        match self.progress.get_ref().deref() {
            Some(p) => { p.set_position(pos) }
            None => { }
        }
    }

    #[inline]
    /// Sets the current message of the progress bar
    pub fn set_progress_bar_message(&self, msg: impl Into<Cow<'static, str>>) {
        match self.progress.get_ref().deref() {
            Some(p) => { p.set_message(msg) }
            None => { }
        }
    }

    /// Write datas to the file.
    /// * `data` - Data
    pub fn write(&self, data: &[u8]) -> Result<(), DownloaderError> {
        match self.file.get_mut().deref_mut() {
            Some(f) => { f.write_all(data)? }
            None => {}
        }
        Ok(())
    } 
}

/// A file downloader
pub struct Downloader<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName> {
    /// internal type
    downloader: Arc<DownloaderInternal<T>>,
}

impl Downloader<LocalFile> {
    /// Create a new [Downloader] instance
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(url: U, headers: H, path: Option<&P>, overwrite: Option<bool>) -> Result<DownloaderResult<Self>, DownloaderError> {
        Ok(match DownloaderInternal::<LocalFile>::new(url, headers, path, overwrite)? {
            DownloaderResult::Ok(d) => {
                DownloaderResult::Ok(Self { downloader: Arc::new(d) })
            }
            DownloaderResult::Canceled => { DownloaderResult::Canceled }
        })
    }
}

#[doc(hidden)]
macro_rules! define_downloader_fn {
    {$f:ident, $t:ty, $doc:expr} => {
        #[inline]
        #[doc = $doc]
        pub fn $f(&self) -> $t {
            self.downloader.$f()
        }
    }
}

impl <T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + 'static> Downloader<T> {
    /// Start download if download not started.
    /// 
    /// Returns the status of the Downloader
    pub fn download(&self) -> DownloaderStatus {
        if !self.is_created() {
            return self.downloader.get_status();
        }
        self.downloader.set_downloading();
        tokio::spawn(check_tasks(Arc::clone(&self.downloader)));
        self.downloader.get_status()
    }

    /// Wait the downloader.
    pub async fn join(&self) -> Result<(), DownloaderError> {
        loop {
            if !self.is_downloading() {
                break;
            }
            tokio::time::sleep(Duration::new(0, 1_000_000)).await;
        }
        Ok(())
    }

    #[inline]
    /// Disable progress bar
    pub fn disable_progress_bar(&self) {
        self.downloader.disable_progress_bar()
    }

    #[inline]
    /// Enable the progress bar
    /// * `style` - The style of the progress bar
    /// * `mults` - The instance of [MultiProgress] if multiple progress bars are needed.
    pub fn enable_progress_bar(&self, style: ProgressStyle, mults: Option<&MultiProgress>) {
        self.downloader.enable_progress_bar(style, mults)
    }

    /// Handle options
    /// * `mults` - The instance of [MultiProgress] if multiple progress bars are needed.
    pub fn handle_options(&self, helper: &OptHelper, mults: Option<Arc<MultiProgress>>) {
        if helper.use_progress_bar() {
            let style = ProgressStyle::default_bar()
                .template(helper.progress_bar_template().as_ref()).unwrap()
                .progress_chars("#>-");
            match mults {
                Some(v) => { self.enable_progress_bar(style, Some(&v)); }
                None => { self.enable_progress_bar(style, None); }
            }
        } else {
            self.disable_progress_bar();
        }
    }
    define_downloader_fn!(is_created, bool, "Returns true if the downloader is created just now.");
    define_downloader_fn!(is_downloading, bool, "Returns true if the downloader is downloading now.");
    define_downloader_fn!(is_multi_threads, bool, "Returns true if is multiple thread mode.");
    define_downloader_fn!(is_downloaded, bool, "Returns true if the downloader is downloaded complete.");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_downloader() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://i.pximg.net/img-original/img/2022/06/12/23/49/43/99014872_p0.png";
    let pb = p.join("99014872_p0.png");
    let downloader = Downloader::<LocalFile>::new(url, json::object!{"referer": "https://www.pixiv.net/"}, Some(&pb), Some(true)).unwrap();
    match downloader {
        DownloaderResult::Ok(v) => {
            assert_eq!(v.is_created(), true);
            v.disable_progress_bar();
            v.download();
            v.join().await.unwrap();
            assert_eq!(v.is_downloaded(), true);
        }
        DownloaderResult::Canceled => { panic!("This should not happened.") }
    }
}
