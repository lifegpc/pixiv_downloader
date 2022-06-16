use std::fmt::Display;

#[derive(Debug)]
/// The result when try create a new [super::Downloader] interface
pub enum DownloaderResult<T> {
    /// Created successfully
    Ok(T),
    /// The target file already downloaded and overwrite is disabled.
    Canceled,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
/// The status of the [super::Downloader]
pub enum DownloaderStatus {
    /// The downloader is just created
    Created,
    /// The downloader is downloading now
    Downloading,
    /// The downloader is downloaded complete
    Downloaded,
    /// The downloader is stoped
    Panic,
}

impl Display for DownloaderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Created => "Created",
            Self::Downloading => "Downloading",
            Self::Downloaded => "Downloaded",
            Self::Panic => "Panic",
        })
    }
}
