use json::JsonValue;
use std::fmt::Debug;

/// To represent a fanbox item
pub struct FanboxItem {
    /// JSON object
    pub data: JsonValue,
}

impl FanboxItem {
    /// Returns the id of the item
    pub fn id(&self) -> Option<u64> {
        let id = &self.data["id"];
        match id.as_u64() {
            Some(id) => Some(id),
            None => match id.as_str() {
                Some(id) => match id.trim().parse::<u64>() {
                    Ok(id) => Some(id),
                    Err(_) => None,
                },
                None => None,
            },
        }
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.data["isLiked"].as_bool()
    }

    #[inline]
    /// Returns item's title
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }
}

impl Debug for FanboxItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxItem")
            .field("id", &self.id())
            .field("title", &self.title())
            .field("is_liked", &self.is_liked())
            .finish_non_exhaustive()
    }
}

impl From<JsonValue> for FanboxItem {
    fn from(data: JsonValue) -> Self {
        Self { data }
    }
}

impl From<&JsonValue> for FanboxItem {
    fn from(data: &JsonValue) -> Self {
        Self::from(data.clone())
    }
}
