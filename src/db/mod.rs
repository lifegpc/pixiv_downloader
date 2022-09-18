pub mod config;
#[cfg(feature = "db_sqlite")]
pub mod sqlite;
pub mod traits;

pub use config::PixivDownloaderDbConfig;
#[cfg(feature = "db_sqlite")]
pub use sqlite::SqliteError;
pub use traits::PixivDownloaderDb;
pub type PixivDownloaderDbError = Box<dyn std::fmt::Display + Send + Sync>;

#[cfg(feature = "db_sqlite")]
impl From<SqliteError> for PixivDownloaderDbError {
    fn from(e: SqliteError) -> Self {
        Box::new(e)
    }
}

#[cfg(not(feature = "db_sqlite"))]
compile_error!("No database backend is enabled.");
