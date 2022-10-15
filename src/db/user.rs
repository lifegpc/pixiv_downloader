use bytes::BytesMut;

use crate::ext::json::ToJson2;

#[derive(Clone)]
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

impl ToJson2 for User {
    fn to_json2(&self) -> json::JsonValue {
        json::object! {
            "id": self.id,
            "name": self.name.as_str(),
            "username": self.username.as_str(),
            "is_admin": self.is_admin,
        }
    }
}
