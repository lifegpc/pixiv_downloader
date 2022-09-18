use super::super::{
    PixivDownloaderDb, PixivDownloaderDbConfig, PixivDownloaderDbError, PixivDownloaderSqliteConfig,
};
use super::SqliteError;
use rusqlite::{Connection, OpenFlags};
use std::sync::Mutex;

pub struct PixivDownloaderSqlite {
    db: Mutex<Connection>,
}

impl PixivDownloaderSqlite {
    fn _new(cfg: &PixivDownloaderSqliteConfig) -> Result<Self, SqliteError> {
        let con = Connection::open_with_flags(
            &cfg.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )?;
        Ok(Self {
            db: Mutex::new(con),
        })
    }
}

impl PixivDownloaderDb for PixivDownloaderSqlite {
    #[allow(unreachable_patterns)]
    fn new<R: AsRef<PixivDownloaderDbConfig> + ?Sized>(
        cfg: &R,
    ) -> Result<Self, PixivDownloaderDbError> {
        match cfg.as_ref() {
            PixivDownloaderDbConfig::Sqlite(cfg) => {
                let db = Self::_new(cfg)?;
                Ok(db)
            }
            _ => panic!("Config mismatched."),
        }
    }
}
