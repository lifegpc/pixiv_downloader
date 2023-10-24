#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum SqliteError {
    DbError(rusqlite::Error),
    DatabaseVersionTooNew,
    UserNameAlreadyExists,
    #[cfg(feature = "serde_json")]
    SerdeError(serde_json::Error),
}
