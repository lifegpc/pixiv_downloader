use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::fanbox_api_test;
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
    pub fn comment_count(&self) -> Option<u64> {
        self.data["commentCount"].as_u64()
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
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
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn image_for_share(&self) -> Option<&str> {
        self.data["imageForShare"].as_str()
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
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    #[inline]
    pub fn next_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["nextPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn prev_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["prevPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
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

impl Debug for FanboxPostArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostArticle")
            .field("id", &self.id())
            .field("comment_count", &self.comment_count())
            .field("cover_image_url", &self.cover_image_url())
            .field("creator_id", &self.creator_id())
            .field("excerpt", &self.excerpt())
            .field("fee_required", &self.fee_required())
            .field("has_adult_content", &self.has_adult_content())
            .field("image_for_share", &self.image_for_share())
            .field("is_liked", &self.is_liked())
            .field("is_restricted", &self.is_restricted())
            .field("like_count", &self.like_count())
            .field("next_post", &self.next_post())
            .field("prev_post", &self.prev_post())
            .field("published_datetime", &self.published_datetime())
            .field("title", &self.title())
            .field("updated_datetime", &self.updated_datetime())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}

/// A reference to another post
pub struct FanboxPostRef {
    /// Raw data
    pub data: JsonValue,
    /// The api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostRef {
    pub async fn get_post(&self) -> Option<FanboxPost> {
        match self.id() {
            Some(id) => match self.client.get_post_info(id).await {
                Some(s) => Some(FanboxPost::new(&s["body"], Arc::clone(&self.client))),
                None => None,
            },
            None => None,
        }
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
}

impl Debug for FanboxPostRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostRef")
            .field("id", &self.id())
            .field("published_datetime", &self.published_datetime())
            .field("title", &self.title())
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

fanbox_api_test!(test_get_post_info, {
    match client.get_post_info(4070200).await {
        Some(data) => match data {
            FanboxPost::Article(data) => {
                println!("{:?}", data);
                match data.next_post() {
                    Some(r) => {
                        println!("{:?}", r);
                        match r.get_post().await {
                            Some(r) => {
                                println!("{:?}", r);
                            }
                            None => {
                                panic!("Failed to get next post.")
                            }
                        }
                    }
                    None => {}
                }
            }
            FanboxPost::Unknown(data) => {
                println!("{}", data);
            }
        },
        None => {
            panic!("Failed to get the post info(第253話 【壁紙プレゼント】ひんやりレモンころねちゃん🍋).");
        }
    }
});
