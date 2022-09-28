use chrono::{DateTime, Utc};

/// A token in the database
pub struct Token {
    /// The token ID
    pub id: u64,
    /// The user ID of the token
    pub user_id: u64,
    /// The token
    pub token: [u8; 64],
    /// The token's creation time
    pub created_at: DateTime<Utc>,
    /// The token's expiration time
    pub expired_at: DateTime<Utc>,
}
