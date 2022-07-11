use super::check::CheckUnknown;
use super::error::FanboxAPIError;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;

/// To represent a fanbox item
pub struct FanboxItem {
    /// JSON object
    pub data: JsonValue,
}

macro_rules! quick_return {
    ($exp:expr) => {
        match $exp {
            Some(r) => {
                return Some(r);
            }
            None => {}
        }
    };
}

impl FanboxItem {
    #[inline]
    pub fn author_name(&self) -> Option<&str> {
        self.data["user"]["name"].as_str()
    }

    #[inline]
    pub fn comment_count(&self) -> Option<u64> {
        self.data["commentCount"].as_u64()
    }

    pub fn cover_image_url(&self) -> Option<&str> {
        quick_return!(self.data["coverImageUrl"].as_str());
        match self.cover_type() {
            Some(t) => {
                if t == "cover_image" {
                    self.cover_url()
                } else {
                    None
                }
            }
            None => None,
        }
    }

    #[inline]
    pub fn cover_type(&self) -> Option<&str> {
        self.data["cover"]["type"].as_str()
    }

    #[inline]
    pub fn cover_url(&self) -> Option<&str> {
        self.data["cover"]["url"].as_str()
    }

    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn excerpt(&self) -> Option<&str> {
        self.data["excerpt"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.data["feeRequired"].as_u64()
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.data["hasAdultContent"].as_bool()
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
    pub fn is_restricted(&self) -> Option<bool> {
        self.data["isRestricted"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.data["likeCount"].as_u64()
    }

    #[inline]
    pub fn published_datetime(&self) -> Option<&str> {
        self.data["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn tags(&self) -> Option<Vec<&str>> {
        let mut list = Vec::new();
        let tags = &self.data["tags"];
        if tags.is_array() {
            for i in tags.members() {
                match i.as_str() {
                    Some(tag) => {
                        list.push(tag);
                    }
                    None => {
                        return None;
                    }
                }
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    /// Returns item's title
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    pub fn updated_datetime(&self) -> Option<&str> {
        self.data["updatedDatetime"].as_str()
    }

    #[inline]
    pub fn user_icon_url(&self) -> Option<&str> {
        self.data["user"]["iconUrl"].as_str()
    }

    #[inline]
    /// Returns pixiv user id
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["user"]["userId"])
    }
}

impl CheckUnknown for FanboxItem {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "commentCount"+,
            "coverImageUrl",
            "cover": [
                "type"+,
                "url",
            ],
            "creatorId"+,
            "excerpt"+,
            "feeRequired"+,
            "hasAdultContent"+,
            "isLiked"+,
            "isRestricted"+,
            "likeCount"+,
            "publishedDatetime"+,
            "tags"+,
            "title"+,
            "updatedDatetime"+,
            "user": [
                "name"+author_name,
                "iconUrl",
                "id"+,
            ],
        );
        Ok(())
    }
}

impl Debug for FanboxItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxItem")
            .field("id", &self.id())
            .field("author_name", &self.author_name())
            .field("comment_count", &self.comment_count())
            .field("cover_image_url", &self.cover_image_url())
            .field("cover_type", &self.cover_type())
            .field("cover_url", &self.cover_url())
            .field("creator_id", &self.creator_id())
            .field("excerpt", &self.excerpt())
            .field("fee_required", &self.fee_required())
            .field("has_adult_content", &self.has_adult_content())
            .field("is_liked", &self.is_liked())
            .field("is_restricted", &self.is_restricted())
            .field("like_count", &self.like_count())
            .field("published_datetime", &self.published_datetime())
            .field("tags", &self.tags())
            .field("title", &self.title())
            .field("updated_datetime", &self.updated_datetime())
            .field("user_icon_url", &self.user_icon_url())
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
