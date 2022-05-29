use super::downloader::Downloader;
use std::io::Seek;
use std::io::Write;

#[derive(Debug)]
/// The result when try create a new [Downloader] interface
pub enum DownloaderResult<T: Write + Seek> {
    /// Created successfully
    Ok(Downloader<T>),
    /// The target file already downloaded and overwrite is disabled.
    Canceled,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
/// The status of the [Downloader]
pub enum DownloaderStatus {
    /// The downloader is just created
    Created,
    /// The downloader is downloading now
    Downloading,
    /// The downloader is downloaded complete
    Downloaded,
}
