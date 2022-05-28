use crate::downloader::pd_file::PdFileError;

#[derive(Debug, derive_more::From)]
pub enum DownloaderError {
    ReqwestError(reqwest::Error),
    PdFileError(PdFileError),
    IoError(std::io::Error),
}
