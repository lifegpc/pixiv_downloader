use super::PixivDownloaderDbConfig;
use super::PixivDownloaderDbError;
#[cfg(feature = "server")]
use super::{Token, User};
use chrono::{DateTime, Utc};

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
    /// Add a new token
    /// * `user_id` - The user ID
    /// * `token` - The token
    /// * `created_at` - The token's expiration time
    /// * `expired_at` - The token's creation time
    /// # Note
    /// if a token with the same user_id already exists, must return None
    async fn add_token(
        &self,
        user_id: u64,
        token: &[u8; 64],
        created_at: &DateTime<Utc>,
        expired_at: &DateTime<Utc>,
    ) -> Result<Option<Token>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Add a new user to database.
    /// * `name` - User name
    /// * `username` - Unique user name
    /// * `password` - Hashed password
    /// * `is_admin` - Whether the user is an admin
    /// # Note
    /// If username already exists, this function must be failed.
    async fn add_user(
        &self,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Delete a user
    /// * `id` - User ID
    /// # Note
    /// All tokens of the user will be deleted.
    async fn delete_user(&self, id: u64) -> Result<bool, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get token by ID
    /// * `id` - The token ID
    async fn get_token(&self, id: u64) -> Result<Option<Token>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get a user by ID
    /// * `id`: The user's ID
    async fn get_user(&self, id: u64) -> Result<Option<User>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get a user by username
    /// * `username`: The user's username
    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, PixivDownloaderDbError>;
    /// Initialize the database (create tables, migrate data, etc.)
    async fn init(&self) -> Result<(), PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// List users
    /// * `offset` - The offset of the first user
    /// * `limit` - The maximum number of users to return
    async fn list_users(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<User>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// List users' id
    /// * `offset` - The offset of the list
    /// * `count` - The maximum count of the list
    async fn list_users_id(
        &self,
        offset: u64,
        count: u64,
    ) -> Result<Vec<u64>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Remove all expired tokens
    /// Return the number of removed tokens
    async fn revoke_expired_tokens(&self) -> Result<usize, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Set a user's information by ID
    /// * `id`: The user's ID
    /// * `name`: The user's name
    /// * `username`: The user's username
    /// * `password`: The user's hashed password
    /// * `is_admin`: Whether the user is an admin
    /// # Note
    /// If the user does not exist, a new user will be created and `id` must not be applied.  
    /// If the user's `username` is taked by another user, the operation must failed.
    async fn set_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Update a user's information
    /// * `id`: The user's ID
    /// * `name`: The user's name
    /// * `username`: The user's username
    /// * `password`: The user's hashed password
    /// * `is_admin`: Whether the user is an admin
    /// # Note
    /// If the user does not exist, the operation must failed.
    /// If the user's `username` is taked by another user, the operation must failed.
    async fn update_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Update a user's name
    /// * `id`: The user's ID
    /// * `name`: The user's name
    async fn update_user_name(&self, id: u64, name: &str) -> Result<User, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Update a user's password
    /// * `id`: The user's ID
    /// * `password`: The user's hashed password
    /// * `token_id`: The token ID
    /// # Note
    /// All tokens of the user except token_id will be revoked
    async fn update_user_password(
        &self,
        id: u64,
        password: &[u8],
        token_id: u64,
    ) -> Result<User, PixivDownloaderDbError>;
}
