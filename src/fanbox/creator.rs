use super::error::FanboxAPIError;
use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use std::convert::From;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

/// Profile image
pub struct FanboxProfileImage {
    /// Raw data
    pub data: JsonValue,
    /// Fanbox API Client
    client: Arc<FanboxClientInternal>,
}

impl FanboxProfileImage {
    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.data["id"].as_str()
    }

    #[inline]
    pub fn image_url(&self) -> Option<&str> {
        self.data["imageUrl"].as_str()
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    #[inline]
    pub fn thumbnail_url(&self) -> Option<&str> {
        self.data["thumbnailUrl"].as_str()
    }
}

impl Debug for FanboxProfileImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxProfileImage")
            .field("id", &self.id())
            .field("image_url", &self.image_url())
            .field("thumbnail_url", &self.thumbnail_url())
            .finish_non_exhaustive()
    }
}

/// Profile item
#[derive(Debug)]
pub enum FanboxProfileItem {
    /// Image
    Image(FanboxProfileImage),
    /// Other types
    Unknown(JsonValue),
}

impl FanboxProfileItem {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        match data["type"].as_str() {
            Some(s) => match s {
                "image" => Self::Image(FanboxProfileImage::new(data, client)),
                _ => Self::Unknown(data.clone()),
            },
            _ => Self::Unknown(data.clone()),
        }
    }
}

/// A list of [FanboxProfileItem]
pub struct FanboxProfileItems {
    /// The list
    list: Vec<FanboxProfileItem>,
}

impl FanboxProfileItems {
    #[inline]
    /// Create a new instance
    pub fn new(
        data: &JsonValue,
        client: Arc<FanboxClientInternal>,
    ) -> Result<Self, FanboxAPIError> {
        if data.is_array() {
            let mut list = Vec::new();
            for i in data.members() {
                list.push(FanboxProfileItem::new(i, Arc::clone(&client)));
            }
            Ok(Self { list })
        } else {
            Err(FanboxAPIError::from("Failed to get profile items."))
        }
    }
}

impl Debug for FanboxProfileItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.list.fmt(f)
    }
}

impl Deref for FanboxProfileItems {
    type Target = Vec<FanboxProfileItem>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for FanboxProfileItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

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

    #[inline]
    pub fn has_booth_shop(&self) -> Option<bool> {
        self.data["hasBoothShop"].as_bool()
    }

    #[inline]
    pub fn is_accepting_request(&self) -> Option<bool> {
        self.data["isAcceptingRequest"].as_bool()
    }

    #[inline]
    pub fn is_followed(&self) -> Option<bool> {
        self.data["isFollowed"].as_bool()
    }

    #[inline]
    pub fn is_stopped(&self) -> Option<bool> {
        self.data["isStopped"].as_bool()
    }

    #[inline]
    pub fn is_supported(&self) -> Option<bool> {
        self.data["isSupported"].as_bool()
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    #[inline]
    pub fn profile_items(&self) -> Result<FanboxProfileItems, FanboxAPIError> {
        FanboxProfileItems::new(&self.data["profileItems"], Arc::clone(&self.client))
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
            .field("has_booth_shop", &self.has_booth_shop())
            .field("is_accepting_request", &self.is_accepting_request())
            .field("is_followed", &self.is_followed())
            .field("is_stopped", &self.is_stopped())
            .field("is_supported", &self.is_supported())
            .field("profile_items", &self.profile_items())
            .field("profile_links", &self.profile_links())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}
