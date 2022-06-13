use crate::downloader::pd_file::PdFileError;
use crate::gettext;
use http::status::StatusCode;
use tokio::time::error::Elapsed;
use tokio::task::JoinError;
use std::fmt::Display;

/// File downloader's error
#[derive(Debug, derive_more::From)]
pub enum DownloaderError {
    /// Request error
    ReqwestError(reqwest::Error),
    /// [PdFileError]
    PdFileError(PdFileError),
    /// Io Error
    IoError(std::io::Error),
    /// String type message
    String(String),
    /// HTTP Error
    ErrorStatusCode(StatusCode),
    /// Timed out
    Timeout(Elapsed),
    /// Failed to join
    JoinError(JoinError),
}

impl Display for DownloaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestError(e) => {
                f.write_str(gettext("Errors occured when requesting: "))?;
                e.fmt(f)
            }
            Self::PdFileError(e) => {
                f.write_str(gettext("Errors occured when operating pd file: "))?;
                e.fmt(f)
            }
            Self::IoError(e) => {
                f.write_str(gettext("Errors occured when operating files: "))?;
                e.fmt(f)
            }
            Self::String(e) => {
                f.write_str(e)
            }
            Self::ErrorStatusCode(e) => {
                f.write_str("HTTP ERROR ")?;
                e.fmt(f)
            }
            Self::Timeout(e) => {
                e.fmt(f)
            }
            Self::JoinError(e) => {
                e.fmt(f)
            }
        }
    }
}

impl From<&str> for DownloaderError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}
