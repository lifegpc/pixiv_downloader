use super::super::check::CheckUnkown;
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

impl CheckUnkown for FanboxArticleParagraphBoldStyle {
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

#[derive(proc_macros::CheckUnkown, Debug)]
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

impl CheckUnkown for FanboxArticleParagraphBlock {
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

#[derive(proc_macros::CheckUnkown, Debug)]
pub enum FanboxArticleBlock {
    Paragraph(FanboxArticleParagraphBlock),
    Unknown(JsonValue),
}

impl FanboxArticleBlock {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        match data["type"].as_str() {
            Some(t) => match t {
                "p" => Self::Paragraph(FanboxArticleParagraphBlock::new(data)),
                _ => Self::Unknown(data.clone()),
            },
            None => Self::Unknown(data.clone()),
        }
    }
}
