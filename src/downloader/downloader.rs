use super::pd_file::PdFile;
use super::pd_file::PdFileResult;
use super::enums::DownloaderResult;
use super::enums::DownloaderStatus;
use super::error::DownloaderError;
use super::local_file::LocalFile;
use super::tasks::create_download_tasks_simple;
use crate::ext::atomic::AtomicQuick;
use crate::ext::io::ClearFile;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::try_err::TryErr;
use crate::utils::ask_need_overwrite;
use crate::webclient::WebClient;
use crate::webclient::ToHeaders;
use reqwest::IntoUrl;
use std::collections::HashMap;
use std::fs::remove_file;
use std::io::Seek;
use std::io::Write;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use tokio::task::JoinHandle;
use url::Url;

#[derive(Debug)]
/// A file downloader
pub struct DownloaderInternal<T: Write + Seek + Send + Sync + ClearFile> {
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
    tasks: RwLock<Vec<JoinHandle<Result<(), DownloaderError>>>>,
    /// Whether to enable mulitple thread mode
    multi: AtomicBool,
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
        }))
    }
}

impl <T: Write + Seek + Send + Sync + ClearFile> DownloaderInternal<T> {
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

    /// Return the status of the downloader.
    pub fn get_status(&self) -> DownloaderStatus {
        self.status.get_ref().clone()
    }

    #[inline]
    /// Returns true if the downloader is created just now.
    pub fn is_created(&self) -> bool {
        *self.status.get_ref() == DownloaderStatus::Created
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
pub struct Downloader<T: Write + Seek + Send + Sync + ClearFile> {
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

impl <T: Write + Seek + Send + Sync + ClearFile + 'static> Downloader<T> {
    /// Start download if download not started.
    /// 
    /// Returns the status of the Downloader
    pub fn download(&self) -> DownloaderStatus {
        if !self.is_created() {
            return self.downloader.get_status();
        }
        if !self.is_multi_threads() {
            let task = tokio::spawn(create_download_tasks_simple(Arc::clone(&self.downloader)));
            self.downloader.add_task(task);
        }
        self.downloader.get_status()
    }

    define_downloader_fn!(is_created, bool, "Returns true if the downloader is created just now.");
    define_downloader_fn!(is_multi_threads, bool, "Returns true if is multiple thread mode.");
}
