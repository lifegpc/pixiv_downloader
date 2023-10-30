use super::enums::DownloaderResult;
use super::enums::DownloaderStatus;
use super::error::DownloaderError;
use super::local_file::LocalFile;
use super::pd_file::PdFile;
use super::pd_file::PdFilePartStatus;
use super::pd_file::PdFileResult;
use super::tasks::check_tasks;
use crate::ext::atomic::AtomicQuick;
use crate::ext::io::ClearFile;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::list::NonTailList;
use crate::opthelper::OptHelper;
use crate::utils::ask_need_overwrite;
use crate::utils::get_file_name_from_url;
use crate::webclient::ToHeaders;
use crate::webclient::WebClient;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
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
use std::ops::Drop;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::RwLock;
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

/// Truncates or extends the underlying file, updating the size of this file.
pub trait SetLen {
    /// Truncates or extends the underlying file, updating the size of this file to become `size`.
    ///
    /// If the `size` is less than the current file’s size, then the file will be shrunk.
    /// If it is greater than the current file’s size, then the file will be extended to `size` and have all of the intermediate data filled in with 0s.
    fn set_len(&mut self, size: u64) -> Result<(), DownloaderError>;
}

/// A file downloader
pub struct DownloaderInternal<
    T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + SetLen,
> {
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
    /// The error message when panic.
    pub error: RwLock<Option<DownloaderError>>,
    /// The maximum retry count. -1 means always retry.
    max_retry_count: AtomicI64,
    /// Retry intervals.
    retry_interval: RwLock<NonTailList<Duration>>,
    /// Current retry count
    retry_count: AtomicI64,
    /// The maximum retry count for each part.
    pub max_part_retry_count: AtomicI64,
    /// The maximun threads to download file.
    pub max_threads: AtomicU64,
    /// The size of the each part when downloading file.
    part_size: AtomicU32,
    /// Is outter [Downloader] dropped
    dropped: AtomicBool,
    /// true if the progress bar has length
    progress_has_length: AtomicBool,
}

impl DownloaderInternal<LocalFile> {
    /// Create a new [DownloaderInternal] instance
    /// * `client` - The web client interface.
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(
        client: Arc<WebClient>,
        url: U,
        headers: H,
        path: Option<&P>,
        overwrite: Option<bool>,
    ) -> Result<DownloaderResult<Self>, DownloaderError> {
        let h = match headers.to_headers() {
            Some(h) => h,
            None => HashMap::new(),
        };
        let mut already_exists = false;
        let pd_file = match path {
            Some(p) => {
                let p = p.as_ref();
                match PdFile::open(p)? {
                    PdFileResult::TargetExisted => match overwrite {
                        Some(overwrite) => {
                            if !overwrite {
                                return Ok(DownloaderResult::Canceled);
                            } else {
                                remove_file(p)?;
                                match PdFile::open(p)? {
                                    PdFileResult::Ok(f) => f,
                                    _ => {
                                        panic!("This should not happened.");
                                    }
                                }
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
                    },
                    PdFileResult::Ok(p) => p,
                    PdFileResult::ExistedOk(p) => {
                        already_exists = true;
                        p
                    }
                }
            }
            None => PdFile::new(),
        };
        let file = match path {
            Some(p) => {
                if already_exists {
                    Some(LocalFile::open(p)?)
                } else {
                    Some(LocalFile::create(p)?)
                }
            }
            None => None,
        };
        let mut l = NonTailList::<Duration>::default();
        l += Duration::new(3, 0);
        Ok(DownloaderResult::Ok(Self {
            client: client,
            pd: Arc::new(pd_file),
            url: Arc::new(url.into_url()?),
            headers: Arc::new(h),
            file: RwLock::new(file),
            status: RwLock::new(DownloaderStatus::Created),
            tasks: RwLock::new(Vec::new()),
            multi: AtomicBool::new(false),
            progress_bar: AtomicBool::new(false),
            progress: RwLock::new(None),
            error: RwLock::new(None),
            max_retry_count: AtomicI64::new(3),
            retry_interval: RwLock::new(l),
            retry_count: AtomicI64::new(0),
            max_part_retry_count: AtomicI64::new(3),
            max_threads: AtomicU64::new(8),
            part_size: AtomicU32::new(0x10000),
            dropped: AtomicBool::new(false),
            progress_has_length: AtomicBool::new(false),
        }))
    }
}

impl<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + SetLen> DownloaderInternal<T> {
    /// Add a new task to tasks
    /// * `task` - Task
    pub fn add_task(&self, task: JoinHandle<Result<(), DownloaderError>>) {
        self.tasks.get_mut().push(task)
    }

    /// Clear all datas in file
    pub fn clear_file(&self) -> std::io::Result<()> {
        match self.file.get_mut().deref_mut() {
            Some(f) => f.clear_file()?,
            None => {}
        };
        Ok(())
    }

    /// Disable the progress bar
    pub fn disable_progress_bar(&self) {
        self.progress_bar.qstore(false);
        self.progress.get_mut().take();
    }

    /// Enable multiple download
    pub fn enable_multiple_download(&self) {
        self.multi.qstore(true);
        if !self.is_multi_threads() {
            log::warn!(
                "{}",
                gettext("Warning: This file will still use single thread mode to download.")
            );
        } else {
            match self.pd.enable_multi() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{}", e);
                }
            }
        }
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
            None => {}
        }
        self.progress_bar.qstore(true);
        self.progress.get_mut().replace(bar);
    }

    #[inline]
    /// Returns true if the progress bar is enabled.
    pub fn enabled_progress_bar(&self) -> bool {
        self.progress_bar.qload()
    }

    /// Fallback to simple thread mode.
    pub fn fallback_to_simp(&self) {
        self.multi.qstore(false);
        match self.pd.disable_multi() {
            Ok(_) => {}
            Err(e) => {
                log::error!("{}", e);
            }
        };
    }

    #[inline]
    /// Finishes the progress bar and sets a message
    pub fn finish_progress_bar_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        match self.progress.get_ref().deref() {
            Some(p) => p.finish_with_message(msg),
            None => {}
        }
    }

    #[inline]
    /// Get the file name from url.
    /// If not available, use `(Unknown)`
    pub fn get_file_name(&self) -> String {
        get_file_name_from_url(self.url.deref().clone())
            .unwrap_or(String::from(gettext("(Unknown)")))
    }

    #[inline]
    /// Get panic error
    pub fn get_panic(&self) -> Option<DownloaderError> {
        self.error.get_mut().take()
    }

    #[inline]
    /// Return the size of the each part
    pub fn get_part_size(&self) -> u32 {
        if self.pd.is_downloading() {
            self.pd.get_part_size()
        } else {
            self.part_size.qload()
        }
    }

    /// Increase the retry count and return the duration should waited.
    /// If [None] is returned, should stop retry.
    pub fn get_retry_duration(&self) -> Option<Duration> {
        let rc = self.retry_count.qload();
        let mc = self.max_retry_count.qload();
        if mc >= 0 && rc >= mc {
            None
        } else {
            let dur = self.retry_interval.get_ref()[mc as usize];
            self.retry_count.qstore(rc + 1);
            Some(dur)
        }
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
            Some(f) => f.get_target_file_name(),
            None => None,
        }
    }

    #[inline]
    /// Advances the position of the progress bar by `delta`
    pub fn inc_progress_bar(&self, delta: u64) {
        match self.progress.get_ref().deref() {
            Some(p) => {
                if !self.progress_has_length.qload() {
                    p.inc_length(delta);
                }
                p.inc(delta);
            }
            None => {}
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
    /// Return true if outter [Downloader] dropped
    pub fn is_dropped(&self) -> bool {
        self.dropped.qload()
    }

    #[inline]
    /// Returns true if the downloader is panic.
    pub fn is_panic(&self) -> bool {
        *self.status.get_ref() == DownloaderStatus::Panic
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
            Some(f) => Ok(f.seek(pos)?),
            None => Ok(0),
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
    /// Set outter [Downloader] dropped
    pub fn set_dropped(&self) {
        self.dropped.qstore(true);
    }

    #[inline]
    /// Truncates or extends the underlying file, updating the size of this file to become `size`.
    ///
    /// If the `size` is less than the current file’s size, then the file will be shrunk.
    /// If it is greater than the current file’s size, then the file will be extended to `size` and have all of the intermediate data filled in with 0s.
    pub fn set_len(&self, size: u64) -> Result<(), DownloaderError> {
        match self.file.get_mut().deref_mut() {
            Some(f) => f.set_len(size)?,
            None => {}
        }
        Ok(())
    }

    #[inline]
    /// Set the downloader is panic and set the error.
    /// * `err` - Error
    pub fn set_panic(&self, err: DownloaderError) {
        self.status.replace_with2(DownloaderStatus::Panic);
        self.error.get_mut().replace(err);
    }

    #[inline]
    /// Set the size of the each part when downloading file
    pub fn set_part_size(&self, part_size: u32) {
        self.part_size.qsave(part_size)
    }

    #[inline]
    /// Sets the length of the progress bar
    pub fn set_progress_bar_length(&self, length: u64) {
        self.progress_has_length.qstore(true);
        match self.progress.get_ref().deref() {
            Some(p) => p.set_length(length),
            None => {}
        }
    }

    #[inline]
    /// Sets the position of the progress bar
    pub fn set_progress_bar_position(&self, pos: u64) {
        match self.progress.get_ref().deref() {
            Some(p) => {
                if !self.progress_has_length.qload() {
                    p.set_length(pos);
                }
                p.set_position(pos);
            }
            None => {}
        }
    }

    #[inline]
    /// Sets the current message of the progress bar
    pub fn set_progress_bar_message(&self, msg: impl Into<Cow<'static, str>>) {
        match self.progress.get_ref().deref() {
            Some(p) => p.set_message(msg),
            None => {}
        }
    }

    #[inline]
    /// Set the maximum retry count. -1 means always.
    pub fn set_max_retry_count(&self, max_retry_count: i64) {
        self.max_retry_count.qstore(max_retry_count)
    }

    #[inline]
    /// Set the maximun threads to download file.
    pub fn set_max_threads(&self, max_threads: u64) {
        self.max_threads.qstore(max_threads)
    }

    #[inline]
    /// Set the retry interval.
    pub fn set_retry_interval(&self, retry_interval: NonTailList<Duration>) {
        self.retry_interval.replace_with2(retry_interval);
    }

    /// Write datas to the file.
    /// * `data` - Data
    pub fn write(&self, data: &[u8]) -> Result<(), DownloaderError> {
        match self.file.get_mut().deref_mut() {
            Some(f) => f.write_all(data)?,
            None => {}
        }
        Ok(())
    }

    /// Write datas to the file.
    /// * `data` - Data
    /// * `pd` - The status of the writed part
    /// * `index` - The index of the writed part
    pub fn write_part(
        &self,
        data: &[u8],
        pd: &Arc<PdFilePartStatus>,
        index: usize,
    ) -> Result<(), DownloaderError> {
        match self.file.get_mut().deref_mut() {
            Some(f) => {
                let offset =
                    (self.get_part_size() as u64) * (index as u64) + (pd.downloaded_size() as u64);
                f.seek(SeekFrom::Start(offset))?;
                f.write_all(data)?;
            }
            None => {}
        }
        Ok(())
    }
}

/// A file downloader
///
/// When dropping downloader, the downloader need some times to let all threads exited.
/// This process is handled after [Drop].
pub struct Downloader<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + SetLen> {
    /// internal type
    downloader: Arc<DownloaderInternal<T>>,
}

impl Downloader<LocalFile> {
    #[inline]
    /// Create a new [Downloader] instance
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(
        url: U,
        headers: H,
        path: Option<&P>,
        overwrite: Option<bool>,
    ) -> Result<DownloaderResult<Self>, DownloaderError> {
        Self::new2(
            Arc::new(WebClient::default()),
            url,
            headers,
            path,
            overwrite,
        )
    }

    /// Create a new [Downloader] instance
    /// * `client` - The web client interface.
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new2<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(
        client: Arc<WebClient>,
        url: U,
        headers: H,
        path: Option<&P>,
        overwrite: Option<bool>,
    ) -> Result<DownloaderResult<Self>, DownloaderError> {
        Ok(
            match DownloaderInternal::<LocalFile>::new(client, url, headers, path, overwrite)? {
                DownloaderResult::Ok(d) => DownloaderResult::Ok(Self {
                    downloader: Arc::new(d),
                }),
                DownloaderResult::Canceled => DownloaderResult::Canceled,
            },
        )
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

impl<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + SetLen + 'static>
    Downloader<T>
{
    /// Start download if download not started.
    ///
    /// Returns the status of the Downloader
    pub fn download(&self) -> DownloaderStatus {
        if !self.is_created() {
            return self.downloader.get_status();
        }
        if !self.downloader.is_downloading() && self.is_multi_threads() {
            match self
                .downloader
                .pd
                .set_part_size(self.downloader.get_part_size())
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{}", e);
                }
            }
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
    /// Enable multiple download
    pub fn enable_multiple_download(&self) {
        self.downloader.enable_multiple_download()
    }

    #[inline]
    /// Enable the progress bar
    /// * `style` - The style of the progress bar
    /// * `mults` - The instance of [MultiProgress] if multiple progress bars are needed.
    pub fn enable_progress_bar(&self, style: ProgressStyle, mults: Option<&MultiProgress>) {
        self.downloader.enable_progress_bar(style, mults)
    }

    /// Get panic error
    pub fn get_panic(self) -> Option<DownloaderError> {
        self.downloader.get_panic()
    }

    /// Handle options
    /// * `mults` - The instance of [MultiProgress] if multiple progress bars are needed.
    pub fn handle_options(&self, helper: &OptHelper, mults: Option<Arc<MultiProgress>>) {
        if helper.use_progress_bar() {
            let style = ProgressStyle::default_bar()
                .template(helper.progress_bar_template().as_ref())
                .unwrap()
                .progress_chars("#>-");
            match mults {
                Some(v) => {
                    self.enable_progress_bar(style, Some(&v));
                }
                None => {
                    self.enable_progress_bar(style, None);
                }
            }
        } else {
            self.disable_progress_bar();
        }
        match helper.download_retry() {
            Some(u) => self.set_max_retry_count(u),
            None => {}
        }
        self.set_retry_interval(helper.download_retry_interval());
        match helper.download_part_retry() {
            Some(u) => self.set_max_part_retry_count(u),
            None => {}
        }
        if helper.multiple_threads_download() {
            self.enable_multiple_download()
        }
        self.set_max_threads(helper.max_threads());
        self.set_part_size(helper.part_size());
    }

    #[inline]
    /// Set the maximum retry count for each part. < 0 means always.
    pub fn set_max_part_retry_count(&self, max_part_retry_count: i64) {
        self.downloader
            .max_part_retry_count
            .qstore(max_part_retry_count)
    }

    #[inline]
    /// Set the maximum retry count. < 0 means always.
    pub fn set_max_retry_count(&self, max_retry_count: i64) {
        self.downloader.set_max_retry_count(max_retry_count)
    }

    #[inline]
    /// Set the maximun threads to download file.
    pub fn set_max_threads(&self, max_threads: u64) {
        self.downloader.set_max_threads(max_threads)
    }

    #[inline]
    /// Set the size of the each part when downloading file
    pub fn set_part_size(&self, part_size: u32) {
        self.downloader.set_part_size(part_size)
    }

    #[inline]
    /// Set the retry interval.
    pub fn set_retry_interval(&self, retry_interval: NonTailList<Duration>) {
        self.downloader.set_retry_interval(retry_interval)
    }
    define_downloader_fn!(
        is_created,
        bool,
        "Returns true if the downloader is created just now."
    );
    define_downloader_fn!(
        is_downloading,
        bool,
        "Returns true if the downloader is downloading now."
    );
    define_downloader_fn!(
        is_multi_threads,
        bool,
        "Returns true if is multiple thread mode."
    );
    define_downloader_fn!(
        is_downloaded,
        bool,
        "Returns true if the downloader is downloaded complete."
    );
    define_downloader_fn!(is_panic, bool, "Returns true if the downloader is panic.");
}

impl<T: Write + Seek + Send + Sync + ClearFile + GetTargetFileName + SetLen> Drop
    for Downloader<T>
{
    fn drop(&mut self) {
        self.downloader.set_dropped();
        #[cfg(test)]
        {
            println!("The downloader is dropped.");
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_downloader() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://i.pximg.net/img-original/img/2022/06/12/23/49/43/99014872_p0.png";
    let pb = p.join("99014872_p0.png");
    {
        let mut file_name = pb.file_name().unwrap().to_owned();
        file_name.push(".pd");
        let mut pdf = pb.clone();
        pdf.set_file_name(file_name);
        if pdf.exists() {
            remove_file(&pdf).unwrap();
        }
        LocalFile::create(&pdf).unwrap();
        assert!(pdf.exists());
    }
    let downloader = Downloader::<LocalFile>::new(
        url,
        json::object! {"referer": "https://www.pixiv.net/"},
        Some(&pb),
        Some(true),
    )
    .unwrap();
    match downloader {
        DownloaderResult::Ok(v) => {
            assert_eq!(v.is_created(), true);
            v.disable_progress_bar();
            v.download();
            v.join().await.unwrap();
            assert_eq!(v.is_downloaded(), true);
        }
        DownloaderResult::Canceled => {
            panic!("This should not happened.")
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_failed_downloader() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://a.com/ssdassaodasdas";
    let pb = p.join("addd");
    let client = Arc::new(WebClient::default());
    let mut retry_interval = NonTailList::<Duration>::default();
    retry_interval += Duration::new(0, 0);
    client
        .get_retry_interval_as_mut()
        .replace(retry_interval.clone());
    client.set_retry(1);
    let downloader =
        Downloader::<LocalFile>::new2(client, url, None, Some(&pb), Some(true)).unwrap();
    match downloader {
        DownloaderResult::Ok(v) => {
            v.set_retry_interval(retry_interval);
            v.set_max_retry_count(1);
            v.set_max_part_retry_count(1);
            assert_eq!(v.is_created(), true);
            v.disable_progress_bar();
            v.download();
            v.join().await.unwrap();
            assert_eq!(v.is_panic(), true);
        }
        DownloaderResult::Canceled => {
            panic!("This should not happened.")
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_downloader_dropped() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://i.pximg.net/img-original/img/2022/06/15/14/44/22/99066680_p0.png";
    let pb = p.join("99066680_p0.png");
    {
        let downloader = Downloader::<LocalFile>::new(
            url,
            json::object! {"referer": "https://www.pixiv.net/"},
            Some(&pb),
            Some(true),
        )
        .unwrap();
        match downloader {
            DownloaderResult::Ok(v) => {
                assert_eq!(v.is_created(), true);
                v.disable_progress_bar();
                v.download();
            }
            DownloaderResult::Canceled => {
                panic!("This should not happened.")
            }
        }
    }
    tokio::time::sleep(Duration::new(1, 0)).await;
    {
        println!("The new downloader is created.");
        let downloader = Downloader::<LocalFile>::new(
            url,
            json::object! {"referer": "https://www.pixiv.net/"},
            Some(&pb),
            Some(false),
        )
        .unwrap();
        match downloader {
            DownloaderResult::Ok(v) => {
                assert_eq!(v.is_created(), true);
                v.disable_progress_bar();
                v.download();
                v.join().await.unwrap();
                assert_eq!(v.is_downloaded(), true);
            }
            DownloaderResult::Canceled => {
                println!("The file is already downloaded. (Too fast network. QaQ)");
            }
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_multi_downloader() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://i.pximg.net/img-original/img/2022/06/18/08/19/18/99124570_p0.jpg";
    let pb = p.join("99124570_p0.jpg");
    {
        let mut file_name = pb.file_name().unwrap().to_owned();
        file_name.push(".pd");
        let mut pdf = pb.clone();
        pdf.set_file_name(file_name);
        if pdf.exists() {
            remove_file(&pdf).unwrap();
        }
        LocalFile::create(&pdf).unwrap();
        assert!(pdf.exists());
    }
    let downloader = Downloader::<LocalFile>::new(
        url,
        json::object! {"referer": "https://www.pixiv.net/"},
        Some(&pb),
        Some(true),
    )
    .unwrap();
    match downloader {
        DownloaderResult::Ok(v) => {
            assert_eq!(v.is_created(), true);
            v.disable_progress_bar();
            v.enable_multiple_download();
            v.download();
            v.join().await.unwrap();
            assert_eq!(v.is_downloaded(), true);
        }
        DownloaderResult::Canceled => {
            panic!("This should not happened.")
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_failed_multi_downloader() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let url = "https://a.com/ssdassaodasdas";
    let pb = p.join("addds");
    let client = Arc::new(WebClient::default());
    let mut retry_interval = NonTailList::<Duration>::default();
    retry_interval += Duration::new(0, 0);
    client
        .get_retry_interval_as_mut()
        .replace(retry_interval.clone());
    client.set_retry(1);
    let downloader =
        Downloader::<LocalFile>::new2(client, url, None, Some(&pb), Some(true)).unwrap();
    match downloader {
        DownloaderResult::Ok(v) => {
            v.set_retry_interval(retry_interval);
            v.set_max_retry_count(1);
            v.set_max_part_retry_count(1);
            v.enable_multiple_download();
            assert_eq!(v.is_created(), true);
            v.disable_progress_bar();
            v.download();
            v.join().await.unwrap();
            assert_eq!(v.is_panic(), true);
        }
        DownloaderResult::Canceled => {
            panic!("This should not happened.")
        }
    }
}
