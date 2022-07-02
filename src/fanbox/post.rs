use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use std::fmt::Debug;
use std::sync::Arc;

/// Fanbox post's article
pub struct FanboxPostArticle {
    /// Raw data
    pub data: JsonValue,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostArticle {
    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.data["feeRequired"].as_u64()
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
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
    pub fn published_datetime(&self) -> Option<&str> {
        self.data["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    pub fn updated_datetime(&self) -> Option<&str> {
        self.data["updatedDatetime"].as_str()
    }
}

impl Debug for FanboxPostArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostArticle")
            .field("id", &self.id())
            .field("cover_image_url", &self.cover_image_url())
            .field("fee_required", &self.fee_required())
            .field("published_datetime", &self.published_datetime())
            .field("title", &self.title())
            .field("updated_datetime", &self.updated_datetime())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
/// The fanbox's post
pub enum FanboxPost {
    /// Article
    Article(FanboxPostArticle),
    /// Unknown
    Unknown(JsonValue),
}

impl FanboxPost {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        match data["type"].as_str() {
            Some(s) => match s {
                "article" => Self::Article(FanboxPostArticle::new(data, client)),
                _ => Self::Unknown(data.clone()),
            },
            _ => Self::Unknown(data.clone()),
        }
    }
}
