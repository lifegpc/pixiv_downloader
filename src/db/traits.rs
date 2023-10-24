use super::PixivDownloaderDbConfig;
use super::PixivDownloaderDbError;
use super::{PixivArtwork, PixivArtworkLock};
#[cfg(feature = "server")]
use super::{PushConfig, PushTask, PushTaskConfig};
#[cfg(feature = "server")]
use super::{Token, User};
use chrono::{DateTime, Utc};
use flagset::FlagSet;

#[async_trait]
pub trait PixivDownloaderDb {
    /// Create a new instance of database
    /// * `cfg` - The database configuration
    fn new<R: AsRef<PixivDownloaderDbConfig> + ?Sized>(
        cfg: &R,
    ) -> Result<Self, PixivDownloaderDbError>
    where
        Self: Sized + Send + Sync;
    /// Add/Update an artwork to the database
    /// * `id` - The artwork ID
    /// * `title` - The artwork title
    /// * `author` - The artwork author
    /// * `uid` - The author's ID
    /// * `description` - The artwork description
    /// * `count` - The artwork's page count
    /// * `is_nsfw` - Whether the artwork is NSFW
    /// * `lock` - Specify which part should not be updated.
    async fn add_pixiv_artwork(
        &self,
        id: u64,
        title: &str,
        author: &str,
        uid: u64,
        description: &str,
        count: u64,
        is_nsfw: bool,
        lock: &FlagSet<PixivArtworkLock>,
    ) -> Result<PixivArtwork, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Add a push task
    /// * `config` - The task's config
    /// * `push_configs` - The task's push configurations
    /// * `ttl` - The task's update interval
    async fn add_push_task(
        &self,
        config: &PushTaskConfig,
        push_configs: &[PushConfig],
        ttl: u64,
    ) -> Result<PushTask, PixivDownloaderDbError>;
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
    /// Delete a token
    /// * `id` - The token ID
    async fn delete_token(&self, id: u64) -> Result<(), PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Delete a user
    /// * `id` - User ID
    /// # Note
    /// All tokens of the user will be deleted.
    async fn delete_user(&self, id: u64) -> Result<bool, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Extend a token's expiration time
    /// * `id` - The token ID
    /// * `expired_at` - The token's new expiration time
    async fn extend_token(
        &self,
        id: u64,
        expired_at: &DateTime<Utc>,
    ) -> Result<(), PixivDownloaderDbError>;
    /// Get a config from database
    /// * `key` - The config key
    async fn get_config(&self, key: &str) -> Result<Option<String>, PixivDownloaderDbError>;
    /// Get a config from database or set a default value to database and return
    /// * `key` - The config key
    /// * `default` - The function to return default value
    async fn get_config_or_set_default(
        &self,
        key: &str,
        default: fn() -> Result<String, PixivDownloaderDbError>,
    ) -> Result<String, PixivDownloaderDbError>;
    /// Get an artwork from database
    /// * `id` - The artwork ID
    async fn get_pixiv_artwork(
        &self,
        id: u64,
    ) -> Result<Option<PixivArtwork>, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Get proxy pixiv secrets
    async fn get_proxy_pixiv_secrets(&self) -> Result<String, PixivDownloaderDbError> {
        self.get_config_or_set_default("proxy_pixiv_secrets", || {
            let mut buf = [0; 32];
            openssl::rand::rand_bytes(&mut buf)?;
            Ok(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &buf,
            ))
        })
        .await
    }
    #[cfg(feature = "server")]
    /// Get a push task by ID
    /// * `id` - The task's ID
    async fn get_push_task(&self, id: u64) -> Result<Option<PushTask>, PixivDownloaderDbError>;
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
    /// Set a config
    /// * `key` - The config key
    /// * `value` - The config value
    async fn set_config(&self, key: &str, value: &str) -> Result<(), PixivDownloaderDbError>;
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
    /// Update a push task
    /// * `id`: The task's ID
    /// * `config`: The task's config
    /// * `push_configs`: The task's push configurations
    /// * `ttl`: The task's update interval
    async fn update_push_task(
        &self,
        id: u64,
        config: Option<&PushTaskConfig>,
        push_configs: Option<&[PushConfig]>,
        ttl: Option<u64>,
    ) -> Result<PushTask, PixivDownloaderDbError>;
    #[cfg(feature = "server")]
    /// Update a push task's last updated time
    /// * `id`: The task's ID
    /// * `last_updated`: The task's last updated time
    async fn update_push_task_last_updated(
        &self,
        id: u64,
        last_updated: &DateTime<Utc>,
    ) -> Result<(), PixivDownloaderDbError>;
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
