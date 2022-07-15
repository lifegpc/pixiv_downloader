use super::super::check::CheckUnknown;
use super::super::error::FanboxAPIError;
use crate::fanbox_api::FanboxClientInternal;
use json::JsonValue;
use proc_macros::check_json_keys;
use proc_macros::create_fanbox_download_helper;
use std::fmt::Debug;
use std::sync::Arc;

pub struct FanboxArticleImage {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleImage {
    #[inline]
    pub fn extension(&self) -> Option<&str> {
        self.data["extension"].as_str()
    }

    #[inline]
    pub fn height(&self) -> Option<u64> {
        self.data["height"].as_u64()
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

    #[inline]
    pub fn original_url(&self) -> Option<&str> {
        self.data["originalUrl"].as_str()
    }

    create_fanbox_download_helper!(original_url);

    #[inline]
    pub fn thumbnail_url(&self) -> Option<&str> {
        self.data["thumbnailUrl"].as_str()
    }

    create_fanbox_download_helper!(thumbnail_url);

    #[inline]
    pub fn width(&self) -> Option<u64> {
        self.data["width"].as_u64()
    }
}

impl CheckUnknown for FanboxArticleImage {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "extension"+,
            "id"+,
            "width"+,
            "height"+,
            "originalUrl"+,
            "thumbnailUrl",
        );
        Ok(())
    }
}

impl Debug for FanboxArticleImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleImage")
            .field("id", &self.id())
            .field("extension", &self.extension())
            .field("height", &self.height())
            .field("original_url", &self.original_url())
            .field("thumbnail_url", &self.thumbnail_url())
            .field("width", &self.width())
            .finish_non_exhaustive()
    }
}

pub struct FanboxArticleImageMap {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleImageMap {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    pub fn get_image<S: AsRef<str> + ?Sized>(&self, id: &S) -> Option<FanboxArticleImage> {
        let id = id.as_ref();
        let image = &self.data[id];
        if image.is_object() {
            Some(FanboxArticleImage::new(image, Arc::clone(&self.client)))
        } else {
            None
        }
    }
}

impl CheckUnknown for FanboxArticleImageMap {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        for (key, _) in self.data.entries() {
            match self.get_image(key) {
                Some(i) => {
                    i.check_unknown()?;
                }
                None => {}
            }
        }
        Ok(())
    }
}

impl Debug for FanboxArticleImageMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("FanboxArticleImageMap");
        for (key, _) in self.data.entries() {
            s.field(key, &self.get_image(key));
        }
        s.finish_non_exhaustive()
    }
}
