use crate::parser::json::parse_u64;
use json::JsonValue;
use std::fmt::Debug;

/// To represent a fanbox item
pub struct FanboxItem {
    /// JSON object
    pub data: JsonValue,
}

impl FanboxItem {
    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.data["feeRequired"].as_u64()
    }

    #[inline]
    /// Returns the id of the item
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.data["isLiked"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.data["likeCount"].as_u64()
    }

    #[inline]
    /// Returns item's title
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    /// Returns pixiv user id
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["user"]["userId"])
    }
}

impl Debug for FanboxItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxItem")
            .field("id", &self.id())
            .field("creator_id", &self.creator_id())
            .field("fee_required", &self.fee_required())
            .field("is_liked", &self.is_liked())
            .field("like_count", &self.like_count())
            .field("title", &self.title())
            .field("user_id", &self.user_id())
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
