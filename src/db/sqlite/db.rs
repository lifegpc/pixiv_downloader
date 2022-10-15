use super::super::{
    PixivDownloaderDb, PixivDownloaderDbConfig, PixivDownloaderDbError, PixivDownloaderSqliteConfig,
};
#[cfg(feature = "server")]
use super::super::{Token, User};
use super::SqliteError;
use bytes::BytesMut;
use chrono::{DateTime, Utc};
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
const FILES_TABLE: &'static str = "CREATE TABLE files (
id INTEGER PRIMARY KEY AUTOINCREMENT,
path TEXT,
last_modified DATETIME,
etag TEXT,
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
count INT
);";
const PIXIV_FILES_TABLE: &'static str = "CREATE TABLE pixiv_files (
id INT,
file_id INT,
page INT
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
const VERSION: [u8; 4] = [1, 0, 0, 4];

pub struct PixivDownloaderSqlite {
    db: Mutex<Connection>,
}

impl PixivDownloaderSqlite {
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
        t.commit()?;
        Ok(())
    }

    #[cfg(feature = "server")]
    fn _delete_user(tx: &Transaction, id: u64) -> Result<bool, SqliteError> {
        let af = tx.execute("DELETE FROM users WHERE id = ?;", [id])?;
        tx.execute("DELETE FROM token WHERE user_id = ?;", [id])?;
        Ok(af > 0)
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
    fn _revoke_expired_tokens(ts: &Transaction) -> Result<usize, SqliteError> {
        let now = Utc::now();
        Ok(ts.execute("DELETE FROM token WHERE expired_at < ?;", [now])?)
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
    async fn delete_user(&self, id: u64) -> Result<bool, PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let tx = db.transaction()?;
        let re = Self::_delete_user(&tx, id)?;
        tx.commit()?;
        Ok(re)
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
    async fn revoke_expired_tokens(&self) -> Result<usize, PixivDownloaderDbError> {
        let mut db = self.db.lock().await;
        let mut tx = db.transaction()?;
        let size = Self::_revoke_expired_tokens(&mut tx)?;
        tx.commit()?;
        Ok(size)
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
