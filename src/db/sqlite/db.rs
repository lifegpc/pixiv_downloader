use super::super::{
    PixivDownloaderDb, PixivDownloaderDbConfig, PixivDownloaderDbError, PixivDownloaderSqliteConfig,
};
use super::SqliteError;
use futures_util::lock::Mutex;
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;

const AUTHORS_TABLE: &'static str = "CREATE TABLE authors (
id INT,
name TEXT,
creator_id TEXT,
icon INT,
big_icon INT,
background INT,
comment TEXT,
webpage TEXT,
PRIMARY KEY (id)
);";
const FILES_TABLE: &'static str = "CREATE TABLE files (
id INT,
path TEXT,
last_modified DATETIME,
etag TEXT,
url TEXT,
PRIMARY KEY (id)
);";
const PIXIV_ARTWORK_TAGS_TABLE: &'static str = "CREATE TABLE pixiv_artwork_tags (
id INT,
tag_id INT,
);";
const PIXIV_ARTWORKS_TABLE: &'static str = "CREATE TABLE pixiv_artworks (
id INT,
title TEXT,
author TEXT,
uid INT,
description TEXT,
count INT,
);";
const PIXIV_FILES_TABLE: &'static str = "CREATE TABLE pixiv_files (
id INT,
file_id INT,
page INT,
);";
const TAGS_TABLE: &'static str = "CREATE TABLE tags (
id INT,
name TEXT,
PRIMARY KEY (id)
);";
const TAGS_I18N_TABLE: &'static str = "CREATE TABLE tags_i18n (
id INT,
lang TEXT,
translated TEXT,
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
    /// Check if the database needed create all tables.
    async fn _check_database(&self) -> Result<bool, SqliteError> {
        let tables = self._get_exists_table().await?;
        let db_version = if tables.contains_key("version") {
            self._read_version().await?
        } else {
            None
        };
        let db_version = match db_version {
            Some(v) => v,
            None => {
                return Ok(false);
            }
        };
        if db_version > VERSION {
            return Err(SqliteError::DatabaseVersionTooNew);
        }
        Ok(true)
    }

    /// Create tables
    async fn _create_table(&self) -> Result<(), SqliteError> {
        let tables = self._get_exists_table().await?;
        if !tables.contains_key("version") {
            self.db.lock().await.execute(VERSION_TABLE, [])?;
            self._write_version().await?;
        }
        if !tables.contains_key("authors") {
            self.db.lock().await.execute(AUTHORS_TABLE, [])?;
        }
        if !tables.contains_key("files") {
            self.db.lock().await.execute(FILES_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_artwork_tags") {
            self.db.lock().await.execute(PIXIV_ARTWORK_TAGS_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_artworks") {
            self.db.lock().await.execute(PIXIV_ARTWORKS_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_files") {
            self.db.lock().await.execute(PIXIV_FILES_TABLE, [])?;
        }
        if !tables.contains_key("tags") {
            self.db.lock().await.execute(TAGS_TABLE, [])?;
        }
        if !tables.contains_key("tags_i18n") {
            self.db.lock().await.execute(TAGS_I18N_TABLE, [])?;
        }
        Ok(())
    }

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

    async fn _read_version(&self) -> Result<Option<[u8; 4]>, SqliteError> {
        let con = self.db.lock().await;
        let mut stmt = con.prepare("SELECT v1, v2, v3, v4 FROM version WHERE id='main';")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some([row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?]))
        } else {
            Ok(None)
        }
    }

    async fn _write_version(&self) -> Result<(), SqliteError> {
        let con = self.db.lock().await;
        let mut stmt = con.prepare(
            "INSERT OR REPLACE INTO INTO version (id, v1, v2, v3, v4) VALUES ('main', ?, ?, ?, ?);",
        )?;
        stmt.execute([VERSION[0], VERSION[1], VERSION[2], VERSION[3]])?;
        Ok(())
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

    /// Optimize the database
    pub async fn vacuum(&self) -> Result<(), SqliteError> {
        self.db.lock().await.execute("VACUUM;", [])?;
        Ok(())
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
        if !self._check_database().await? {
            self._create_table().await?;
        }
        Ok(())
    }
}
