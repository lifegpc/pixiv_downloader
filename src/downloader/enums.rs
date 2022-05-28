use super::downloader::Downloader;

#[derive(Debug)]
/// The result when try create a new [Downloader] interface
pub enum DownloaderResult {
    /// Created successfully
    Ok(Downloader),
    /// The target file already downloaded and overwrite is disabled.
    Canceled,
}
