#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum SqliteError {
    DbError(rusqlite::Error),
    DatabaseVersionTooNew,
    UserNameAlreadyExists,
}
