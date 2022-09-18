use super::super::{
    PixivDownloaderDb, PixivDownloaderDbConfig, PixivDownloaderDbError, PixivDownloaderSqliteConfig,
};
use super::SqliteError;
use futures_util::lock::Mutex;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::collections::HashMap;

const TAGS_TABLE: &'static str = "CREATE TABLE tags (
id INT,
name TEXT,
PRIMARY KEY (id)
);";
const VERSION_TABLE: &'static str = "CREATE TABLE version (
id TEXT,
v1 INT,
v2 INT,
v3 INT,
v4 INT,
PRIMARY KEY (id)
);";
const VERSION: [u8; 4] = [1, 0, 0, 0];

pub struct PixivDownloaderSqlite {
    db: Mutex<Connection>,
}

impl PixivDownloaderSqlite {
    /// Get all exists tables
    async fn _get_exists_table(&self) -> Result<HashMap<String, ()>, SqliteError> {
        let con = self.db.lock().await;
        let mut tables = HashMap::new();
        let mut stmt = con.prepare("SELECT name FROM main.sqlite_master WHERE type='table';")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            tables.insert(row.get(0)?, ());
        }
        Ok(tables)
    }

    fn _new(cfg: &PixivDownloaderSqliteConfig) -> Result<Self, SqliteError> {
        let db = Connection::open_with_flags(
            &cfg.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )?;
        Ok(Self { db: Mutex::new(db) })
    }
}

#[async_trait]
impl PixivDownloaderDb for PixivDownloaderSqlite {
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

    async fn init(&self) -> Result<(), PixivDownloaderDbError> {
        Ok(())
    }
}
