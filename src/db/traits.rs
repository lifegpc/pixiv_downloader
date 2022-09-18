use super::PixivDownloaderDbConfig;
use super::PixivDownloaderDbError;

#[async_trait]
pub trait PixivDownloaderDb {
    /// Create a new instance of database
    fn new<R: AsRef<PixivDownloaderDbConfig> + ?Sized>(
        cfg: &R,
    ) -> Result<Self, PixivDownloaderDbError>
    where
        Self: Sized + Send + Sync;
    /// Initialize the database (create tables, migrate data, etc.)
    async fn init(&self) -> Result<(), PixivDownloaderDbError>;
}
