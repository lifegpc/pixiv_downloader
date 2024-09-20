#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum SqliteError {
    DbError(rusqlite::Error),
    DatabaseVersionTooNew,
    UserNameAlreadyExists,
    SerdeError(serde_json::Error),
    Str(String),
}

impl From<&str> for SqliteError {
    fn from(value: &str) -> Self {
        Self::Str(String::from(value))
    }
}
