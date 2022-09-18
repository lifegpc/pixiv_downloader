pub mod config;
#[cfg(feature = "db_sqlite")]
pub mod sqlite;
pub mod traits;

pub use config::check_db_config;
pub use config::PixivDownloaderDbConfig;
#[cfg(feature = "db_sqlite")]
pub use config::PixivDownloaderSqliteConfig;
#[cfg(feature = "db_sqlite")]
pub use sqlite::{PixivDownloaderSqlite, SqliteError};
pub use traits::PixivDownloaderDb;
pub type PixivDownloaderDbError = Box<dyn std::fmt::Display + Send + Sync>;

use crate::{get_helper, gettext};

#[cfg(feature = "db_sqlite")]
impl From<SqliteError> for PixivDownloaderDbError {
    fn from(e: SqliteError) -> Self {
        Box::new(e)
    }
}

#[cfg(not(feature = "db_sqlite"))]
compile_error!("No database backend is enabled.");

/// Open the database
pub fn open_database() -> Result<Box<dyn PixivDownloaderDb + Send + Sync>, PixivDownloaderDbError> {
    let cfg = get_helper().db();
    if cfg.is_none() {
        return Err(Box::new(String::from(gettext(
            "No database configuration provided.",
        ))));
    }
    #[cfg(feature = "db_sqlite")]
    {
        if matches!(cfg, PixivDownloaderDbConfig::Sqlite(_)) {
            return Ok(Box::new(PixivDownloaderSqlite::new(&cfg)?));
        }
    }
    Err(Box::new(String::from(gettext("Unknown database type."))))
}

/// Open the database and initialize it
pub async fn open_and_init_database(
) -> Result<Box<dyn PixivDownloaderDb + Send + Sync>, PixivDownloaderDbError> {
    let db = open_database()?;
    db.init().await?;
    Ok(db)
}
