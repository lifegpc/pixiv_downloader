use super::pd_file::PdFile;
use super::pd_file::PdFileResult;
use super::error::DownloaderError;
use crate::webclient::WebClient;
use crate::webclient::ToHeaders;
use reqwest::IntoUrl;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use url::Url;

/// A file downloader
pub struct Downloader {
    /// The webclient
    client: Arc<WebClient>,
    /// The download status
    status: Arc<PdFile>,
    /// The url of the file
    url: Arc<Url>,
    /// The HTTP headers map
    headers: Arc<HashMap<String, String>>,
    /// The target file
    file: RwLock<Option<File>>,
}

impl Downloader {
    /// Create a new [Downloader] instance
    /// * `url` - The url of the file
    /// * `header` - HTTP headers
    /// * `path` - The path to store downloaded file.
    /// * `overwrite` - Whether to overwrite file
    pub fn new<U: IntoUrl, H: ToHeaders, P: AsRef<Path> + ?Sized>(url: U, headers: H, path: Option<&P>, overwrite: Option<bool>) -> Result<Self, DownloaderError> {
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
                        // #TODO
                        PdFile::new()
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
                    Some(File::open(p)?)
                } else {
                    Some(File::create(p)?)
                }
            }
            None => { None }
        };
        Ok(Self {
            client: Arc::new(WebClient::new()),
            status: Arc::new(pd_file),
            url: Arc::new(url.into_url()?),
            headers: Arc::new(h),
            file: RwLock::new(file),
        })
    }
}
