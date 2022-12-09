use super::super::check::CheckUnknown;
use super::super::error::FanboxAPIError;
use super::block::FanboxArticleBlock;
use super::file::FanboxArticleFileMap;
use super::image::FanboxArticleImageMap;
use super::url_embed::FanboxArticleUrlEmbedMap;
use crate::fanbox_api::FanboxClientInternal;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;
use std::sync::Arc;

/// Article body
pub struct FanboxArticleBody {
    /// Raw data
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleBody {
    #[inline]
    pub fn blocks(&self) -> Option<Vec<FanboxArticleBlock>> {
        let blocks = &self.data["blocks"];
        if blocks.is_array() {
            let mut list = Vec::new();
            for i in blocks.members() {
                list.push(FanboxArticleBlock::new(i))
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn file_map(&self) -> Option<FanboxArticleFileMap> {
        let map = &self.data["fileMap"];
        if map.is_object() {
            Some(FanboxArticleFileMap::new(map, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn image_map(&self) -> Option<FanboxArticleImageMap> {
        let map = &self.data["imageMap"];
        if map.is_object() {
            Some(FanboxArticleImageMap::new(map, Arc::clone(&self.client)))
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

    #[inline]
    pub fn url_embed_map(&self) -> Option<FanboxArticleUrlEmbedMap> {
        let map = &self.data["urlEmbedMap"];
        if map.is_object() {
            Some(FanboxArticleUrlEmbedMap::new(map, Arc::clone(&self.client)))
        } else {
            None
        }
    }
}

impl CheckUnknown for FanboxArticleBody {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "blocks"+,
            "imageMap"+,
            "fileMap"+,
            "embedMap": [],
            "urlEmbedMap"+,
        );
        match self.blocks() {
            Some(blocks) => {
                for i in blocks {
                    i.check_unknown()?;
                }
            }
            None => {}
        }
        match self.file_map() {
            Some(map) => {
                map.check_unknown()?;
            }
            None => {}
        }
        match self.image_map() {
            Some(map) => {
                map.check_unknown()?;
            }
            None => {}
        }
        match self.url_embed_map() {
            Some(map) => {
                map.check_unknown()?;
            }
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxArticleBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleBody")
            .field("blocks", &self.blocks())
            .field("image_map", &self.image_map())
            .field("url_embed_map", &self.url_embed_map())
            .finish_non_exhaustive()
    }
}
