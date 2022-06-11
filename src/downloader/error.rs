use crate::downloader::pd_file::PdFileError;
use http::status::StatusCode;
use tokio::time::error::Elapsed;

#[derive(Debug, derive_more::From)]
pub enum DownloaderError {
    ReqwestError(reqwest::Error),
    PdFileError(PdFileError),
    IoError(std::io::Error),
    String(String),
    ErrorStatusCode(StatusCode),
    Timeout(Elapsed),
}

impl From<&str> for DownloaderError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}
