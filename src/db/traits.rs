use super::PixivDownloaderDbConfig;
use super::PixivDownloaderDbError;
#[cfg(feature = "server")]
use super::{Token, User};

#[async_trait]
pub trait PixivDownloaderDb {
    /// Create a new instance of database
    /// * `cfg` - The database configuration
    fn new<R: AsRef<PixivDownloaderDbConfig> + ?Sized>(
        cfg: &R,
    ) -> Result<Self, PixivDownloaderDbError>
    where
        Self: Sized + Send + Sync;
    #[cfg(feature = "server")]
    /// Add root user to database.
    /// * `name` - User name
    /// * `username` - Unique user name
    /// * `password` - Hashed password
    async fn add_root_user(
        &self,
        name: &str,
        username: &str,
        password: &[u8],
    ) -> Result<User, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get token by ID
    /// * `id` - The token ID
    async fn get_token(&self, id: u64) -> Result<Option<Token>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get a user by ID
    /// * `id`: The user's ID
    async fn get_user(&self, id: u64) -> Result<Option<User>, PixivDownloaderDbError>;
    /// Initialize the database (create tables, migrate data, etc.)
    async fn init(&self) -> Result<(), PixivDownloaderDbError>;
}
