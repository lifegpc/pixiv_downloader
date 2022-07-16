use super::super::check::CheckUnknown;
use super::super::creator::FanboxCreator;
use super::super::error::FanboxAPIError;
use crate::fanbox_api::FanboxClientInternal;
use json::JsonValue;
use proc_macros::check_json_keys;
use proc_macros::fanbox_api_test;
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
    pub fn profile(&self) -> Option<FanboxCreator> {
        let profile = &self.data["profile"];
        if profile.is_object() {
            Some(FanboxCreator::new(&profile, Arc::clone(&self.client)))
        } else {
            None
        }
    }
}

impl CheckUnknown for FanboxArticleUrlEmbedFanboxCreator {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "type",
            "profile"+,
        );
        match self.profile() {
            Some(profile) => {
                profile.check_unknown()?;
            }
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxArticleUrlEmbedFanboxCreator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleUrlEmbedFanboxCreator")
            .field("id", &self.id())
            .field("profile", &self.profile())
            .finish_non_exhaustive()
    }
}

pub struct FanboxArticleUrlEmbedHTML {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleUrlEmbedHTML {
    #[inline]
    pub fn html(&self) -> Option<&str> {
        self.data["html"].as_str()
    }

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
}

impl CheckUnknown for FanboxArticleUrlEmbedHTML {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "type",
            "id"+,
            "html"+,
        );
        Ok(())
    }
}

impl Debug for FanboxArticleUrlEmbedHTML {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleUrlEmbedHTML")
            .field("id", &self.id())
            .field("html", &self.html())
            .finish_non_exhaustive()
    }
}

#[derive(proc_macros::CheckUnknown, Debug)]
pub enum FanboxArticleUrlEmbed {
    FanboxCreator(FanboxArticleUrlEmbedFanboxCreator),
    HTML(FanboxArticleUrlEmbedHTML),
    HTMLCard(FanboxArticleUrlEmbedHTML),
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
                "html" => Self::HTML(FanboxArticleUrlEmbedHTML::new(data, client)),
                "html.card" => Self::HTMLCard(FanboxArticleUrlEmbedHTML::new(data, client)),
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

fanbox_api_test!(test_html_card, {
    match client.get_post_info(4135937).await {
        Some(post) => {
            assert_eq!(post.title(), Some("第255話 夏の新刊入稿しました！ジュエハキャラバンの話"));
            assert_eq!(post.creator_id(), Some("shiratamaco"));
            println!("{:#?}", post);
            match post.check_unknown() {
                Ok(_) => {}
                Err(e) => {
                    panic!("Check unknown: {}", e);
                }
            }
        }
        None => {
            panic!("Failed to get post(第255話 夏の新刊入稿しました！ジュエハキャラバンの話).")
        }
    }
});
