use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use std::fmt::Debug;
use std::sync::Arc;

/// To present a creator's info
pub struct FanboxCreator {
    /// Raw data
    pub data: JsonValue,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxCreator {
    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.data["description"].as_str()
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.data["hasAdultContent"].as_bool()
    }

    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    #[inline]
    pub fn profile_links(&self) -> Option<Vec<&str>> {
        let mut links = Vec::new();
        let pro = &self.data["profileLinks"];
        if pro.is_array() {
            for i in pro.members() {
                match i.as_str() {
                    Some(s) => links.push(s),
                    None => {
                        return None;
                    }
                }
            }
            Some(links)
        } else {
            None
        }
    }

    #[inline]
    pub fn user_icon_url(&self) -> Option<&str> {
        self.data["user"]["iconUrl"].as_str()
    }

    #[inline]
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["user"]["userId"])
    }

    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.data["user"]["name"].as_str()
    }
}

impl Debug for FanboxCreator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxCreator")
            .field("creator_id", &self.creator_id())
            .field("cover_image_url", &self.cover_image_url())
            .field("description", &self.description())
            .field("has_adult_content", &self.has_adult_content())
            .field("profile_links", &self.profile_links())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}
