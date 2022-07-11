use super::super::check::CheckUnknown;
use super::super::error::FanboxAPIError;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;

pub struct FanboxArticleParagraphBoldStyle {
    pub data: JsonValue,
}

impl FanboxArticleParagraphBoldStyle {
    #[inline]
    pub fn length(&self) -> Option<u64> {
        self.data["length"].as_u64()
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }

    #[inline]
    pub fn offset(&self) -> Option<u64> {
        self.data["offset"].as_u64()
    }
}

impl CheckUnknown for FanboxArticleParagraphBoldStyle {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "offset"+,
            "length"+,
            "type",
        );
        Ok(())
    }
}

impl Debug for FanboxArticleParagraphBoldStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleParagraphBoldStyle")
            .field("length", &self.length())
            .field("offset", &self.offset())
            .finish_non_exhaustive()
    }
}

#[derive(proc_macros::CheckUnknown, Debug)]
pub enum FanboxArticleParagraphStyle {
    Bold(FanboxArticleParagraphBoldStyle),
    Unknown(JsonValue),
}

impl FanboxArticleParagraphStyle {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        match data["type"].as_str() {
            Some(t) => match t {
                "bold" => Self::Bold(FanboxArticleParagraphBoldStyle::new(data)),
                _ => Self::Unknown(data.clone()),
            },
            None => Self::Unknown(data.clone()),
        }
    }
}

pub struct FanboxArticleImageBlock {
    pub data: JsonValue,
}

impl FanboxArticleImageBlock {
    #[inline]
    pub fn image_id(&self) -> Option<&str> {
        self.data["imageId"].as_str()
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }
}

impl CheckUnknown for FanboxArticleImageBlock {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "type",
            "imageId"+,
        );
        Ok(())
    }
}

impl Debug for FanboxArticleImageBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleImageBlock")
            .field("image_id", &self.image_id())
            .finish_non_exhaustive()
    }
}

pub struct FanboxArticleParagraphBlock {
    pub data: JsonValue,
}

impl FanboxArticleParagraphBlock {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }

    #[inline]
    pub fn styles(&self) -> Option<Vec<FanboxArticleParagraphStyle>> {
        let styles = &self.data["styles"];
        if styles.is_array() {
            let mut list = Vec::new();
            for i in styles.members() {
                list.push(FanboxArticleParagraphStyle::new(i))
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn text(&self) -> Option<&str> {
        self.data["text"].as_str()
    }
}

impl CheckUnknown for FanboxArticleParagraphBlock {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "styles",
            "text"+,
            "type",
        );
        match self.styles() {
            Some(styles) => {
                for style in styles {
                    style.check_unknown()?;
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxArticleParagraphBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleParagraphBlock")
            .field("styles", &self.styles())
            .field("text", &self.text())
            .finish_non_exhaustive()
    }
}

pub struct FanboxArticleUrlEmbedBlock {
    pub data: JsonValue,
}

impl FanboxArticleUrlEmbedBlock {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }

    #[inline]
    pub fn url_embed_id(&self) -> Option<&str> {
        self.data["urlEmbedId"].as_str()
    }
}

impl CheckUnknown for FanboxArticleUrlEmbedBlock {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "type",
            "urlEmbedId"+,
        );
        Ok(())
    }
}

impl Debug for FanboxArticleUrlEmbedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxArticleUrlEmbedBlock")
            .field("url_embed_id", &self.url_embed_id())
            .finish_non_exhaustive()
    }
}

#[derive(proc_macros::CheckUnknown, Debug)]
pub enum FanboxArticleBlock {
    Image(FanboxArticleImageBlock),
    Paragraph(FanboxArticleParagraphBlock),
    UrlEmbed(FanboxArticleUrlEmbedBlock),
    Unknown(JsonValue),
}

impl FanboxArticleBlock {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        match data["type"].as_str() {
            Some(t) => match t {
                "image" => Self::Image(FanboxArticleImageBlock::new(data)),
                "p" => Self::Paragraph(FanboxArticleParagraphBlock::new(data)),
                "url_embed" => Self::UrlEmbed(FanboxArticleUrlEmbedBlock::new(data)),
                _ => Self::Unknown(data.clone()),
            },
            None => Self::Unknown(data.clone()),
        }
    }
}
