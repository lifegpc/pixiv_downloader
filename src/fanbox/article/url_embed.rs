use super::super::check::CheckUnknown;
use super::super::error::FanboxAPIError;
use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;
use std::sync::Arc;

pub struct FanboxArticleUrlEmbedFanboxCreator {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleUrlEmbedFanboxCreator {
    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.data["id"].as_str()
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
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["profile"]["user"]["userId"])
    }

    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.data["profile"]["user"]["name"].as_str()
    }
}

impl CheckUnknown for FanboxArticleUrlEmbedFanboxCreator {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "type",
            "profile": [
                "user": [
                    "userId"+user_id,
                    "name"+user_name,
                ],
            ],
        );
        Ok(())
    }
}

impl Debug for FanboxArticleUrlEmbedFanboxCreator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleUrlEmbedFanboxCreator")
            .field("id", &self.id())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}

#[derive(proc_macros::CheckUnknown, Debug)]
pub enum FanboxArticleUrlEmbed {
    FanboxCreator(FanboxArticleUrlEmbedFanboxCreator),
    Unknown(JsonValue),
}

impl FanboxArticleUrlEmbed {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        match data["type"].as_str() {
            Some(t) => match t {
                "fanbox.creator" => {
                    Self::FanboxCreator(FanboxArticleUrlEmbedFanboxCreator::new(data, client))
                }
                _ => Self::Unknown(data.clone()),
            },
            None => Self::Unknown(data.clone()),
        }
    }
}

pub struct FanboxArticleUrlEmbedMap {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleUrlEmbedMap {
    #[inline]
    pub fn get_url_embed<S: AsRef<str> + ?Sized>(&self, id: &S) -> Option<FanboxArticleUrlEmbed> {
        let id = id.as_ref();
        let embed = &self.data[id];
        if embed.is_object() {
            Some(FanboxArticleUrlEmbed::new(embed, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }
}

impl CheckUnknown for FanboxArticleUrlEmbedMap {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        for (key, _) in self.data.entries() {
            match self.get_url_embed(key) {
                Some(embed) => {
                    embed.check_unknown()?;
                }
                None => {}
            }
        }
        Ok(())
    }
}

impl Debug for FanboxArticleUrlEmbedMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("FanboxArticleUrlEmbedMap");
        for (key, _) in self.data.entries() {
            d.field(key, &self.get_url_embed(key));
        }
        d.finish_non_exhaustive()
    }
}
