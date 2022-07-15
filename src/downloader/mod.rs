/// A file downloader
pub mod downloader;
/// The enums of the downloader
pub mod enums;
/// File downloader's error
pub mod error;
/// Downloader helper
pub mod helper;
/// Local file type
pub mod local_file;
/// The pd file
pub mod pd_file;
/// Deal download tasks
pub mod tasks;
pub use downloader::Downloader;
pub use enums::DownloaderResult;
pub use error::DownloaderError;
pub use helper::DownloaderHelper;
pub use local_file::LocalFile;
