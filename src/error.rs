use crate::downloader::DownloaderError;

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum PixivDownloaderError {
    DownloaderError(DownloaderError),
    String(String),
}
