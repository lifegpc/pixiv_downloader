use bytes::BytesMut;

/// A user in the database
pub struct User {
    /// The user ID
    pub id: u64,
    /// The user's name
    pub name: String,
    /// Unique user name
    pub username: String,
    /// hashed password
    pub password: BytesMut,
    /// Whether the user is an admin
    pub is_admin: bool,
}
