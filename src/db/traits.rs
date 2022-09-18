use super::PixivDownloaderDbConfig;
use super::PixivDownloaderDbError;

pub trait PixivDownloaderDb {
    /// Create a new instance of database
    fn new<R: AsRef<PixivDownloaderDbConfig> + ?Sized>(
        cfg: &R,
    ) -> Result<Self, PixivDownloaderDbError>
    where
        Self: Sized;
}
