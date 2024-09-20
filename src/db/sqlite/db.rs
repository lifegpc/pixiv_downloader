#[cfg(feature = "server")]
use super::super::Optional2Extension;
use super::super::{PixivArtwork, PixivArtworkLock};
use super::super::{
    PixivDownloaderDb, PixivDownloaderDbConfig, PixivDownloaderDbError, PixivDownloaderSqliteConfig,
};
#[cfg(feature = "server")]
use super::super::{PushConfig, PushTask, PushTaskConfig};
#[cfg(feature = "server")]
use super::super::{Token, User};
use super::SqliteError;
#[cfg(feature = "server")]
use crate::tmp_cache::TmpCacheEntry;
use bytes::BytesMut;
use chrono::{DateTime, Utc};
use flagset::FlagSet;
use futures_util::lock::Mutex;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};
use std::collections::HashMap;

const AUTHORS_TABLE: &'static str = "CREATE TABLE authors (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT,
creator_id TEXT,
icon INT,
big_icon INT,
background INT,
comment TEXT,
webpage TEXT
);";
const CONFIG_TABLE: &'static str = "CREATE TABLE config (
key TEXT PRIMARY KEY,
value TEXT
);";
const FILES_TABLE: &'static str = "CREATE TABLE files (
id INTEGER PRIMARY KEY AUTOINCREMENT,
path TEXT,
last_modified DATETIME,
url TEXT
);";
const PIXIV_ARTWORK_TAGS_TABLE: &'static str = "CREATE TABLE pixiv_artwork_tags (
id INT,
tag_id INT
);";
const PIXIV_ARTWORKS_TABLE: &'static str = "CREATE TABLE pixiv_artworks (
id INT,
title TEXT,
author TEXT,
uid INT,
description TEXT,
count INT,
is_nsfw BOOLEAN,
lock INT
);";
const PIXIV_FILES_TABLE: &'static str = "CREATE TABLE pixiv_files (
id INT,
file_id INT,
page INT
);";
const PUSH_TASK_TABLE: &'static str = "CREATE TABLE push_task (
id INTEGER PRIMARY KEY AUTOINCREMENT,
config TEXT,
push_configs TEXT,
last_updated DATETIME,
ttl INT
);";
const PUSH_TASK_DATA_TABLE: &'static str = "CREATE TABLE push_task_data (
id INT,
data TEXT,
PRIMARY KEY (id)
);";
const TAGS_TABLE: &'static str = "CREATE TABLE tags (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT
);";
const TAGS_I18N_TABLE: &'static str = "CREATE TABLE tags_i18n (
id INT,
lang TEXT,
translated TEXT
);";
const TMP_CACHE_TABLE: &'static str = "CREATE TABLE tmp_cache (
url TEXT,
path TEXT,
last_used DATETIME,
PRIMARY KEY (url)
);";
const TOKEN_TABLE: &'static str = "CREATE TABLE token (
id INTEGER PRIMARY KEY AUTOINCREMENT,
user_id INT,
token TEXT,
created_at DATETIME,
expired_at DATETIME
);";
const USERS_TABLE: &'static str = "CREATE TABLE users (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT,
username TEXT,
password TEXT,
is_admin BOOLEAN
);";
const VERSION_TABLE: &'static str = "CREATE TABLE version (
id TEXT,
v1 INT,
v2 INT,
v3 INT,
v4 INT,
PRIMARY KEY (id)
);";
const VERSION: [u8; 4] = [1, 0, 0, 9];

pub struct PixivDownloaderSqlite {
    db: Mutex<Connection>,
}

impl PixivDownloaderSqlite {
    fn _add_pixiv_artwork(
        ts: &Transaction,
        id: u64,
        title: &str,
        author: &str,
        uid: u64,
        description: &str,
        count: u64,
        is_nsfw: bool,
        lock: &FlagSet<PixivArtworkLock>,
    ) -> Result<(), SqliteError> {
        ts.execute(
            "INSERT INTO pixiv_artworks (id, title, author, uid, description, count, is_nsfw, lock) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                id,
                title,
                author,
                uid,
                description,
                count,
                is_nsfw,
                lock.bits(),
            ),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _add_push_task(
        ts: &Transaction,
        config: &PushTaskConfig,
        push_configs: &[PushConfig],
        ttl: u64,
    ) -> Result<u64, PixivDownloaderDbError> {
        ts.execute(
            "INSERT INTO push_task (config, push_configs, last_updated, ttl) VALUES (?, ?, ?, ?);",
            (
                serde_json::to_string(config)?,
                serde_json::to_string(push_configs)?,
                DateTime::UNIX_EPOCH,
                ttl,
            ),
        )?;
        Ok(ts.query_row(
            "SELECT seq FROM sqlite_sequence WHERE name = 'push_task';",
            [],
            |row| Ok(row.get(0)?),
        )?)
    }

    #[cfg(feature = "server")]
    async fn _add_root_user(
        &self,
        name: &str,
        username: &str,
        password: &[u8],
    ) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let tx = db.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO users VALUES (0, ?, ?, ?, true);",
            (name, username, password),
        )?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _add_token(
        tx: &Transaction,
        user_id: u64,
        token: &[u8; 64],
        created_at: &DateTime<Utc>,
        expired_at: &DateTime<Utc>,
    ) -> Result<(), SqliteError> {
        tx.execute(
            "INSERT INTO token (user_id, token, created_at, expired_at) VALUES (?, ?, ?, ?);",
            (user_id, token, created_at, expired_at),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _add_user(
        tx: &Transaction,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<(), SqliteError> {
        tx.execute(
            "INSERT INTO users (name, username, password, is_admin) VALUES (?, ?, ?, ?);",
            (name, username, password, is_admin),
        )?;
        Ok(())
    }
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
        if db_version < VERSION {
            {
                let mut lock = self.db.lock().await;
                let tx = lock.transaction()?;
                if db_version < [1, 0, 0, 1] {
                    tx.execute(TOKEN_TABLE, [])?;
                    tx.execute(USERS_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 2] {
                    tx.execute("ALTER TABLE users ADD is_admin BOOLEAN;", [])?;
                }
                if db_version < [1, 0, 0, 3] {
                    tx.execute("DROP TABLE token;", [])?;
                    tx.execute(TOKEN_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 4] {
                    tx.execute("DROP TABLE authors;", [])?;
                    tx.execute("DROP TABLE files;", [])?;
                    tx.execute("DROP TABLE tags;", [])?;
                    tx.execute("DROP TABLE token;", [])?;
                    tx.execute("DROP TABLE users;", [])?;
                    tx.execute(AUTHORS_TABLE, [])?;
                    tx.execute(FILES_TABLE, [])?;
                    tx.execute(TAGS_TABLE, [])?;
                    tx.execute(TOKEN_TABLE, [])?;
                    tx.execute(USERS_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 5] {
                    tx.execute("DROP TABLE files;", [])?;
                    tx.execute(FILES_TABLE, [])?;
                    tx.execute("ALTER TABLE pixiv_artworks ADD is_nsfw BOOLEAN;", [])?;
                    tx.execute("ALTER TABLE pixiv_artworks ADD lock INT;", [])?;
                }
                if db_version < [1, 0, 0, 6] {
                    tx.execute(CONFIG_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 7] {
                    tx.execute(PUSH_TASK_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 8] {
                    tx.execute(PUSH_TASK_DATA_TABLE, [])?;
                }
                if db_version < [1, 0, 0, 9] {
                    tx.execute(TMP_CACHE_TABLE, [])?;
                }
                self._write_version(&tx)?;
                tx.commit()?;
            }
            self.vacuum().await?;
        }
        Ok(true)
    }

    /// Create tables
    async fn _create_table(&self) -> Result<(), SqliteError> {
        let tables = self._get_exists_table().await?;
        let mut lock = self.db.lock().await;
        let t = lock.transaction()?;
        if !tables.contains_key("version") {
            t.execute(VERSION_TABLE, [])?;
            self._write_version(&t)?;
        }
        if !tables.contains_key("authors") {
            t.execute(AUTHORS_TABLE, [])?;
        }
        if !tables.contains_key("files") {
            t.execute(FILES_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_artwork_tags") {
            t.execute(PIXIV_ARTWORK_TAGS_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_artworks") {
            t.execute(PIXIV_ARTWORKS_TABLE, [])?;
        }
        if !tables.contains_key("pixiv_files") {
            t.execute(PIXIV_FILES_TABLE, [])?;
        }
        if !tables.contains_key("push_task") {
            t.execute(PUSH_TASK_TABLE, [])?;
        }
        if !tables.contains_key("tags") {
            t.execute(TAGS_TABLE, [])?;
        }
        if !tables.contains_key("tags_i18n") {
            t.execute(TAGS_I18N_TABLE, [])?;
        }
        if !tables.contains_key("token") {
            t.execute(TOKEN_TABLE, [])?;
        }
        if !tables.contains_key("users") {
            t.execute(USERS_TABLE, [])?;
        }
        if !tables.contains_key("config") {
            t.execute(CONFIG_TABLE, [])?;
        }
        if !tables.contains_key("push_task_data") {
            t.execute(PUSH_TASK_DATA_TABLE, [])?;
        }
        if !tables.contains_key("tmp_cache") {
            t.execute("TMP_CACHE_TABLE", [])?;
        }
        t.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _delete_tmp_cache(tx: &Transaction, url: &str) -> Result<(), SqliteError> {
        tx.execute("DELETE FROM tmp_cache WHERE url = ?;", [url])?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _delete_token(tx: &Transaction, id: u64) -> Result<(), SqliteError> {
        tx.execute("DELETE FROM token WHERE id = ?;", [id])?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _delete_user(tx: &Transaction, id: u64) -> Result<bool, SqliteError> {
        let af = tx.execute("DELETE FROM users WHERE id = ?;", [id])?;
        tx.execute("DELETE FROM token WHERE user_id = ?;", [id])?;
        Ok(af > 0)
    }

    #[cfg(feature = "server")]
    fn _extend_token(
        tx: &Transaction,
        id: u64,
        expired_at: &DateTime<Utc>,
    ) -> Result<(), SqliteError> {
        tx.execute(
            "UPDATE token SET expired_at = ? WHERE id = ?;",
            (expired_at, id),
        )?;
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

    #[cfg(feature = "server")]
    async fn get_all_push_tasks(&self) -> Result<Vec<PushTask>, PixivDownloaderDbError> {
        let con = self.db.lock().await;
        let mut stmt = con.prepare("SELECT * FROM push_task;")?;
        let mut rows = stmt.query([])?;
        let mut tasks = Vec::new();
        while let Some(row) = rows.next()? {
            let config: String = row.get(1)?;
            let config: PushTaskConfig = serde_json::from_str(&config)?;
            let push_configs: String = row.get(2)?;
            let push_configs: Vec<PushConfig> = serde_json::from_str(&push_configs)?;
            tasks.push(PushTask {
                id: row.get(0)?,
                config,
                push_configs,
                last_updated: row.get(3)?,
                ttl: row.get(4)?,
            });
        }
        Ok(tasks)
    }

    async fn get_config(&self, key: &str) -> Result<Option<String>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row("SELECT value FROM config WHERE key = ?;", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    async fn get_pixiv_artwork(&self, id: u64) -> Result<Option<PixivArtwork>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row("SELECT * FROM pixiv_artworks WHERE id = ?;", [id], |row| {
                let lock: u8 = row.get(7)?;
                Ok(PixivArtwork {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    author: row.get(2)?,
                    uid: row.get(3)?,
                    description: row.get(4)?,
                    count: row.get(5)?,
                    is_nsfw: row.get(6)?,
                    lock: match FlagSet::<PixivArtworkLock>::new(lock) {
                        Ok(f) => f,
                        Err(_) => {
                            return Err(rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Integer,
                                "Failed to parse lock bit.".into(),
                            )
                            .into())
                        }
                    },
                })
            })
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn get_push_task(&self, id: u64) -> Result<Option<PushTask>, SqliteError> {
        let con = self.db.lock().await;
        con.query_row_and_then::<PushTask, SqliteError, _, _>(
            "SELECT * FROM push_task WHERE id = ?;",
            [id],
            |row| {
                let config: String = row.get(1)?;
                let config: PushTaskConfig = serde_json::from_str(&config)?;
                let push_configs: String = row.get(2)?;
                let push_configs: Vec<PushConfig> = serde_json::from_str(&push_configs)?;
                Ok(PushTask {
                    id: row.get(0)?,
                    config,
                    push_configs,
                    last_updated: row.get(3)?,
                    ttl: row.get(4)?,
                })
            },
        )
        .optional2()
    }

    #[cfg(feature = "server")]
    async fn get_push_task_data(&self, id: u64) -> Result<Option<String>, PixivDownloaderDbError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row(
                "SELECT data FROM push_task_data WHERE id = ?;",
                [id],
                |row| row.get(0),
            )
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn get_tmp_cache(
        &self,
        url: &str,
    ) -> Result<Option<TmpCacheEntry>, PixivDownloaderDbError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row("SELECT * FROM tmp_cache WHERE url = ?;", [url], |row| {
                Ok(TmpCacheEntry {
                    url: row.get(0)?,
                    path: row.get(1)?,
                    last_used: row.get(2)?,
                })
            })
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn get_tmp_caches(&self, ttl: i64) -> Result<Vec<TmpCacheEntry>, PixivDownloaderDbError> {
        let t = Utc::now()
            .checked_sub_signed(chrono::TimeDelta::seconds(ttl))
            .ok_or(PixivDownloaderDbError::Str(String::from(
                "Failed to calculate expired time by ttl.",
            )))?;
        let con = self.db.lock().await;
        let mut stmt = con.prepare("SELECT * FROM tmp_cache WHERE last_used < ?;")?;
        let mut rows = stmt.query([t])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            entries.push(TmpCacheEntry {
                url: row.get(0)?,
                path: row.get(1)?,
                last_used: row.get(2)?,
            });
        }
        Ok(entries)
    }

    #[cfg(feature = "server")]
    async fn get_token(&self, id: u64) -> Result<Option<Token>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row("SELECT * FROM token WHERE id = ?;", [id], |row| {
                Ok(Token {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    token: row.get(2)?,
                    created_at: row.get(3)?,
                    expired_at: row.get(4)?,
                })
            })
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn _get_token_by_user_id_and_token(
        &self,
        user_id: u64,
        token: &[u8; 64],
    ) -> Result<Option<Token>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row(
                "SELECT * FROM token WHERE user_id = ? AND token = ?;",
                (user_id, token),
                |row| {
                    Ok(Token {
                        id: row.get(0)?,
                        user_id: row.get(1)?,
                        token: row.get(2)?,
                        created_at: row.get(3)?,
                        expired_at: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn _get_user(&self, id: u64) -> Result<Option<User>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row("SELECT * FROM users WHERE id = ?;", [id], |row| {
                let password: Vec<u8> = row.get(3)?;
                let password: &[u8] = &password;
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    username: row.get(2)?,
                    password: BytesMut::from(password),
                    is_admin: row.get(4)?,
                })
            })
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn _get_user_by_username(&self, username: &str) -> Result<Option<User>, SqliteError> {
        let con = self.db.lock().await;
        Ok(con
            .query_row(
                "SELECT * FROM users WHERE username = ?;",
                [username],
                |row| {
                    let password: Vec<u8> = row.get(3)?;
                    let password: &[u8] = &password;
                    Ok(User {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        username: row.get(2)?,
                        password: BytesMut::from(password),
                        is_admin: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    #[cfg(feature = "server")]
    async fn _list_users(&self, offset: u64, limit: u64) -> Result<Vec<User>, SqliteError> {
        let con = self.db.lock().await;
        let mut stmt = con.prepare("SELECT * FROM users LIMIT ?, ?;")?;
        let mut rows = stmt.query([offset, limit])?;
        let mut users = Vec::new();
        while let Some(row) = rows.next()? {
            let password: Vec<u8> = row.get(3)?;
            let password: &[u8] = &password;
            users.push(User {
                id: row.get(0)?,
                name: row.get(1)?,
                username: row.get(2)?,
                password: BytesMut::from(password),
                is_admin: row.get(4)?,
            });
        }
        Ok(users)
    }

    #[cfg(feature = "server")]
    async fn _list_users_id(&self, offset: u64, count: u64) -> Result<Vec<u64>, SqliteError> {
        let con = self.db.lock().await;
        let mut stmt = con.prepare("SELECT id FROM users LIMIT ?, ?;")?;
        let mut rows = stmt.query([offset, count])?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            ids.push(row.get(0)?);
        }
        Ok(ids)
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

    #[cfg(feature = "server")]
    fn _put_tmp_cache(ts: &Transaction, url: &str, path: &str) -> Result<(), SqliteError> {
        let t = Utc::now();
        ts.execute(
            "INSERT INTO tmp_cache (url, path, last_used) VALUES (?, ?, ?);",
            (url, path, t),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _revoke_expired_tokens(ts: &Transaction) -> Result<usize, SqliteError> {
        let now = Utc::now();
        Ok(ts.execute("DELETE FROM token WHERE expired_at < ?;", [now])?)
    }

    fn _set_config(ts: &Transaction, key: &str, value: &str) -> Result<(), SqliteError> {
        ts.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?, ?);",
            (key, value),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _set_push_task_data(
        ts: &Transaction,
        id: u64,
        data: &str,
    ) -> Result<(), PixivDownloaderDbError> {
        ts.execute(
            "INSERT OR REPLACE INTO push_task_data (id, data) VALUES (?, ?);",
            (id, data),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn _set_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, SqliteError> {
        let con = self.db.lock().await;
        let has_user = con
            .query_row("SELECT * FROM users WHERE id = ?;", [id], |_| Ok(()))
            .optional()
            .is_ok();
        let has_username = con
            .query_row(
                "SELECT * FROM users WHERE username = ?;",
                [username],
                |row| {
                    if has_user {
                        let uid: u64 = row.get(0)?;
                        Ok(uid != id)
                    } else {
                        Ok(true)
                    }
                },
            )
            .optional()?
            .unwrap_or(false);
        if !has_user {
            if has_username {
                Err(SqliteError::UserNameAlreadyExists)
            } else {
                con.execute(
                    "INSERT INTO users (name, username, password, is_admin) VALUES (?, ?, ?, ?);",
                    (name, username, password, is_admin),
                )?;
                Ok(con.query_row(
                    "SELECT * FROM users WHERE username = ?;",
                    [username],
                    |row| {
                        let password: Vec<u8> = row.get(3)?;
                        let password: &[u8] = &password;
                        Ok(User {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            username: row.get(2)?,
                            password: BytesMut::from(password),
                            is_admin: row.get(4)?,
                        })
                    },
                )?)
            }
        } else {
            if has_username {
                Err(SqliteError::UserNameAlreadyExists)
            } else {
                con.execute("INSERT OR REPLACE INTO users (id, name, username, password, is_admin) VALUES (?, ?, ?, ?, ?);", (id, name, username, password, is_admin))?;
                Ok(
                    con.query_row("SELECT * FROM users WHERE id = ?;", [id], |row| {
                        let password: Vec<u8> = row.get(3)?;
                        let password: &[u8] = &password;
                        Ok(User {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            username: row.get(2)?,
                            password: BytesMut::from(password),
                            is_admin: row.get(4)?,
                        })
                    })?,
                )
            }
        }
    }

    #[cfg(feature = "server")]
    fn _update_push_task(
        tx: &Transaction,
        id: u64,
        config: Option<&PushTaskConfig>,
        push_configs: Option<&[PushConfig]>,
        ttl: Option<u64>,
    ) -> Result<(), PixivDownloaderDbError> {
        match config {
            Some(config) => {
                let config = serde_json::to_string(config)?;
                tx.execute(
                    "UPDATE push_task SET config = ? WHERE id = ?;",
                    (config, id),
                )?;
            }
            None => {}
        }
        match push_configs {
            Some(push_configs) => {
                let push_configs = serde_json::to_string(push_configs)?;
                tx.execute(
                    "UPDATE push_task SET push_configs = ? WHERE id = ?;",
                    (push_configs, id),
                )?;
            }
            None => {}
        }
        match ttl {
            Some(ttl) => {
                tx.execute("UPDATE push_task SET ttl = ? WHERE id = ?;", (ttl, id))?;
            }
            None => {}
        }
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _update_push_task_last_updated(
        tx: &Transaction,
        id: u64,
        last_updated: &DateTime<Utc>,
    ) -> Result<(), PixivDownloaderDbError> {
        tx.execute(
            "UPDATE push_task SET last_updated = ? WHERE id = ?;",
            (last_updated, id),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _update_tmp_cache(tx: &Transaction, url: &str) -> Result<(), PixivDownloaderDbError> {
        let now = Utc::now();
        tx.execute(
            "UPDATE tmp_cache SET last_used = ? WHERE url = ?;",
            (now, url),
        )?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn _update_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<usize, SqliteError> {
        let con = self.db.lock().await;
        let has_username = con
            .query_row(
                "SELECT * FROM users WHERE username = ?;",
                [username],
                |row| {
                    let uid: u64 = row.get(0)?;
                    Ok(uid != id)
                },
            )
            .optional()?
            .unwrap_or(false);
        if has_username {
            return Err(SqliteError::UserNameAlreadyExists);
        }
        Ok(con.execute(
            "UPDATE users SET name = ?, username = ?, password = ?, is_admin = ? WHERE id = ?;",
            (name, username, password, is_admin, id),
        )?)
    }

    #[cfg(feature = "server")]
    fn _update_user_name(ts: &Transaction, id: u64, name: &str) -> Result<usize, SqliteError> {
        Ok(ts.execute("UPDATE users SET name = ? WHERE id = ?;", (name, id))?)
    }

    #[cfg(feature = "server")]
    fn _update_user_password(
        ts: &Transaction,
        id: u64,
        password: &[u8],
        token_id: u64,
    ) -> Result<(), SqliteError> {
        ts.execute(
            "UPDATE users SET password = ? WHERE id = ?;",
            (password, id),
        )?;
        ts.execute(
            "DELETE FROM token WHERE user_id = ? AND id != ?;",
            (id, token_id),
        )?;
        Ok(())
    }

    fn _write_version<'a>(&self, ts: &Transaction<'a>) -> Result<(), SqliteError> {
        let mut stmt = ts.prepare(
            "INSERT OR REPLACE INTO version (id, v1, v2, v3, v4) VALUES ('main', ?, ?, ?, ?);",
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

#[async_trait(+Sync)]
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
    ) -> Result<PixivArtwork, PixivDownloaderDbError> {
        {
            let mut con = self.db.lock().await;
            let mut ts = con.transaction()?;
            Self::_add_pixiv_artwork(
                &mut ts,
                id,
                title,
                author,
                uid,
                description,
                count,
                is_nsfw,
                lock,
            )?;
            ts.commit()?;
        }
        Ok(self.get_pixiv_artwork(id).await?.expect("User not found:"))
    }

    #[cfg(feature = "server")]
    async fn add_push_task(
        &self,
        config: &PushTaskConfig,
        push_configs: &[PushConfig],
        ttl: u64,
    ) -> Result<PushTask, PixivDownloaderDbError> {
        let task = {
            let mut db = self.db.lock().await;
            let tx = db.transaction()?;
            let task = Self::_add_push_task(&tx, config, push_configs, ttl)?;
            tx.commit()?;
            task
        };
        Ok(self.get_push_task(task).await?.expect("Task not found:"))
    }

    #[cfg(feature = "server")]
    async fn add_root_user(
        &self,
        name: &str,
        username: &str,
        password: &[u8],
    ) -> Result<User, PixivDownloaderDbError> {
        self._add_root_user(name, username, password).await?;
        Ok(self.get_user(0).await?.expect("Root user not found:"))
    }

    #[cfg(feature = "server")]
    async fn add_token(
        &self,
        user_id: u64,
        token: &[u8; 64],
        created_at: &DateTime<Utc>,
        expired_at: &DateTime<Utc>,
    ) -> Result<Option<Token>, PixivDownloaderDbError> {
        if self
            ._get_token_by_user_id_and_token(user_id, token)
            .await?
            .is_some()
        {
            return Ok(None);
        }
        {
            let mut db = self.db.lock().await;
            let mut tx = db.transaction()?;
            Self::_add_token(&mut tx, user_id, token, created_at, expired_at)?;
            tx.commit()?;
        }
        Ok(self._get_token_by_user_id_and_token(user_id, token).await?)
    }

    #[cfg(feature = "server")]
    async fn add_user(
        &self,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError> {
        if self.get_user_by_username(username).await?.is_some() {
            return Err(SqliteError::UserNameAlreadyExists.into());
        }
        {
            let mut db = self.db.lock().await;
            let tx = db.transaction()?;
            Self::_add_user(&tx, name, username, password, is_admin)?;
            tx.commit()?;
        }
        Ok(self
            .get_user_by_username(username)
            .await?
            .expect("User not found:"))
    }

    #[cfg(feature = "server")]
    async fn delete_tmp_cache(&self, url: &str) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        Self::_delete_tmp_cache(&mut tx, url)?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn delete_token(&self, id: u64) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        Self::_delete_token(&mut tx, id)?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn delete_user(&self, id: u64) -> Result<bool, PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let tx = db.transaction()?;
        let re = Self::_delete_user(&tx, id)?;
        tx.commit()?;
        Ok(re)
    }

    #[cfg(feature = "server")]
    async fn extend_token(
        &self,
        id: u64,
        expired_at: &DateTime<Utc>,
    ) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        Self::_extend_token(&mut tx, id, expired_at)?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn get_all_push_tasks(&self) -> Result<Vec<PushTask>, PixivDownloaderDbError> {
        Ok(self.get_all_push_tasks().await?)
    }

    async fn get_config(&self, key: &str) -> Result<Option<String>, PixivDownloaderDbError> {
        Ok(self.get_config(key).await?)
    }

    async fn get_config_or_set_default(
        &self,
        key: &str,
        default: fn() -> Result<String, PixivDownloaderDbError>,
    ) -> Result<String, PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let tx = db.transaction()?;
        let value = tx
            .query_row("SELECT value FROM config WHERE key = ?;", [key], |row| {
                row.get(0)
            })
            .optional()?;
        match value {
            Some(v) => {
                tx.commit()?;
                Ok(v)
            }
            None => {
                let v = default()?;
                Self::_set_config(&tx, key, &v)?;
                tx.commit()?;
                Ok(v)
            }
        }
    }

    async fn get_pixiv_artwork(
        &self,
        id: u64,
    ) -> Result<Option<PixivArtwork>, PixivDownloaderDbError> {
        Ok(self.get_pixiv_artwork(id).await?)
    }

    #[cfg(feature = "server")]
    async fn get_push_task(&self, id: u64) -> Result<Option<PushTask>, PixivDownloaderDbError> {
        Ok(self.get_push_task(id).await?)
    }

    #[cfg(feature = "server")]
    async fn get_push_task_data(&self, id: u64) -> Result<Option<String>, PixivDownloaderDbError> {
        Ok(self.get_push_task_data(id).await?)
    }

    #[cfg(feature = "server")]
    async fn get_tmp_cache(
        &self,
        url: &str,
    ) -> Result<Option<TmpCacheEntry>, PixivDownloaderDbError> {
        Ok(self.get_tmp_cache(url).await?)
    }

    #[cfg(feature = "server")]
    async fn get_tmp_caches(&self, ttl: i64) -> Result<Vec<TmpCacheEntry>, PixivDownloaderDbError> {
        Ok(self.get_tmp_caches(ttl).await?)
    }

    #[cfg(feature = "server")]
    async fn get_token(&self, id: u64) -> Result<Option<Token>, PixivDownloaderDbError> {
        Ok(self.get_token(id).await?)
    }

    #[cfg(feature = "server")]
    async fn get_user(&self, id: u64) -> Result<Option<User>, PixivDownloaderDbError> {
        Ok(self._get_user(id).await?)
    }

    #[cfg(feature = "server")]
    async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, PixivDownloaderDbError> {
        Ok(self._get_user_by_username(username).await?)
    }

    async fn init(&self) -> Result<(), PixivDownloaderDbError> {
        if !self._check_database().await? {
            self._create_table().await?;
        }
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn list_users(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<User>, PixivDownloaderDbError> {
        Ok(self._list_users(offset, limit).await?)
    }

    #[cfg(feature = "server")]
    async fn list_users_id(
        &self,
        offset: u64,
        count: u64,
    ) -> Result<Vec<u64>, PixivDownloaderDbError> {
        Ok(self._list_users_id(offset, count).await?)
    }

    #[cfg(feature = "server")]
    async fn put_tmp_cache(&self, url: &str, path: &str) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        let size = Self::_put_tmp_cache(&mut tx, url, path)?;
        tx.commit()?;
        Ok(size)
    }

    #[cfg(feature = "server")]
    async fn revoke_expired_tokens(&self) -> Result<usize, PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        let size = Self::_revoke_expired_tokens(&mut tx)?;
        tx.commit()?;
        Ok(size)
    }

    async fn set_config(&self, key: &str, value: &str) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        Self::_set_config(&mut tx, key, value)?;
        tx.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn set_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError> {
        Ok(self
            ._set_user(id, name, username, password, is_admin)
            .await?)
    }

    #[cfg(feature = "server")]
    async fn update_push_task(
        &self,
        id: u64,
        config: Option<&PushTaskConfig>,
        push_configs: Option<&[PushConfig]>,
        ttl: Option<u64>,
    ) -> Result<PushTask, PixivDownloaderDbError> {
        {
            let mut db = self.db.lock().await;
            let tx = db.transaction()?;
            Self::_update_push_task(&tx, id, config, push_configs, ttl)?;
            tx.commit()?;
        }
        Ok(self.get_push_task(id).await?.expect("Task not found:"))
    }

    #[cfg(feature = "server")]
    async fn set_push_task_data(&self, id: u64, data: &str) -> Result<(), PixivDownloaderDbError> {
        {
            let mut db = self.db.lock().await;
            let tx = db.transaction()?;
            Self::_set_push_task_data(&tx, id, data)?;
            tx.commit()?;
        }
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn update_push_task_last_updated(
        &self,
        id: u64,
        last_updated: &DateTime<Utc>,
    ) -> Result<(), PixivDownloaderDbError> {
        {
            let mut db = self.db.lock().await;
            let mut tx = db.transaction()?;
            Self::_update_push_task_last_updated(&mut tx, id, last_updated)?;
            tx.commit()?;
        }
        Ok(())
    }

    #[cfg(feature = "server")]
    async fn update_tmp_cache(&self, url: &str) -> Result<(), PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        let size = Self::_update_tmp_cache(&mut tx, url)?;
        tx.commit()?;
        Ok(size)
    }

    #[cfg(feature = "server")]
    async fn update_user(
        &self,
        id: u64,
        name: &str,
        username: &str,
        password: &[u8],
        is_admin: bool,
    ) -> Result<User, PixivDownloaderDbError> {
        self._update_user(id, name, username, password, is_admin)
            .await?;
        Ok(self.get_user(id).await?.expect("User not found:"))
    }

    #[cfg(feature = "server")]
    async fn update_user_name(&self, id: u64, name: &str) -> Result<User, PixivDownloaderDbError> {
        {
            let mut db = self.db.lock().await;
            let mut tx = db.transaction()?;
            Self::_update_user_name(&mut tx, id, name)?;
            tx.commit()?;
        }
        Ok(self.get_user(id).await?.expect("User not found:"))
    }

    #[cfg(feature = "server")]
    async fn update_user_password(
        &self,
        id: u64,
        password: &[u8],
        token_id: u64,
    ) -> Result<User, PixivDownloaderDbError> {
        {
            let mut db = self.db.lock().await;
            let mut tx = db.transaction()?;
            Self::_update_user_password(&mut tx, id, password, token_id)?;
            tx.commit()?;
        }
        Ok(self.get_user(id).await?.expect("User not found:"))
    }
}
