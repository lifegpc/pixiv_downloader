pub mod config;
pub mod pixiv_artworks;
#[cfg(feature = "server")]
pub mod push_task;
#[cfg(feature = "db_sqlite")]
pub mod sqlite;
#[cfg(feature = "server")]
pub mod token;
pub mod traits;
#[cfg(feature = "server")]
pub mod user;

pub use config::check_db_config;
pub use config::PixivDownloaderDbConfig;
#[cfg(feature = "db_sqlite")]
pub use config::PixivDownloaderSqliteConfig;
pub use pixiv_artworks::{PixivArtwork, PixivArtworkLock};
#[cfg(feature = "server")]
pub use push_task::{PushConfig, PushTask, PushTaskConfig};
#[cfg(feature = "db_sqlite")]
pub use sqlite::{PixivDownloaderSqlite, SqliteError};
#[cfg(feature = "server")]
pub use token::Token;
pub use traits::PixivDownloaderDb;
#[cfg(feature = "server")]
pub use user::User;

#[derive(Debug, derive_more::Display)]
pub enum PixivDownloaderDbError {
    AnyHow(anyhow::Error),
    #[cfg(feature = "db_sqlite")]
    Sqlite(SqliteError),
}

impl PixivDownloaderDbError {
    pub fn msg<S: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>(msg: S) -> Self {
        Self::AnyHow(anyhow::Error::msg(msg))
    }
}

impl<T> From<T> for PixivDownloaderDbError
where
    T: Into<anyhow::Error>,
{
    fn from(e: T) -> Self {
        Self::AnyHow(e.into())
    }
}

use crate::gettext;

#[cfg(feature = "db_sqlite")]
impl From<SqliteError> for PixivDownloaderDbError {
    fn from(e: SqliteError) -> Self {
        PixivDownloaderDbError::Sqlite(e)
    }
}

#[cfg(all(feature = "db_sqlite", feature = "server"))]
pub trait Optional2Extension<T, E> {
    fn optional2(self) -> Result<Option<T>, E>;
}

#[cfg(all(feature = "db_sqlite", feature = "server"))]
impl<T> Optional2Extension<T, PixivDownloaderDbError> for Result<T, PixivDownloaderDbError> {
    fn optional2(self) -> Result<Option<T>, PixivDownloaderDbError> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e {
                PixivDownloaderDbError::Sqlite(e) => match e {
                    SqliteError::DbError(e) => match e {
                        rusqlite::Error::QueryReturnedNoRows => Ok(None),
                        _ => Err(PixivDownloaderDbError::Sqlite(SqliteError::DbError(e))),
                    },
                    _ => Err(PixivDownloaderDbError::Sqlite(e)),
                },
                _ => Err(e),
            },
        }
    }
}

#[cfg(all(feature = "db_sqlite", feature = "server"))]
impl<T> Optional2Extension<T, SqliteError> for Result<T, SqliteError> {
    fn optional2(self) -> Result<Option<T>, SqliteError> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e {
                SqliteError::DbError(e) => match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    _ => Err(SqliteError::DbError(e)),
                },
                _ => Err(e),
            },
        }
    }
}

#[cfg(not(feature = "db_sqlite"))]
compile_error!("No database backend is enabled.");

/// Open the database
pub fn open_database(
    cfg: PixivDownloaderDbConfig,
) -> Result<Box<dyn PixivDownloaderDb + Send + Sync>, PixivDownloaderDbError> {
    if cfg.is_none() {
        return Err(PixivDownloaderDbError::msg(String::from(gettext(
            "No database configuration provided.",
        ))));
    }
    #[cfg(feature = "db_sqlite")]
    {
        if matches!(cfg, PixivDownloaderDbConfig::Sqlite(_)) {
            return Ok(Box::new(PixivDownloaderSqlite::new(&cfg)?));
        }
    }
    Err(PixivDownloaderDbError::msg(String::from(gettext(
        "Unknown database type.",
    ))))
}

/// Open the database and initialize it
pub async fn open_and_init_database(
    cfg: PixivDownloaderDbConfig,
) -> Result<Box<dyn PixivDownloaderDb + Send + Sync>, PixivDownloaderDbError> {
    let db = open_database(cfg)?;
    db.init().await?;
    Ok(db)
}
