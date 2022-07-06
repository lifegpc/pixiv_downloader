use super::comment::FanboxComment;
use crate::fanbox_api::FanboxClientInternal;
use json::JsonValue;
use std::fmt::Debug;
use std::sync::Arc;

/// A list of fanbox item.
pub struct FanboxCommentList {
    /// Item list
    pub items: Vec<FanboxComment>,
    /// The url of the next page
    next_url: Option<String>,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxCommentList {
    /// Create a new instance
    pub fn new(value: &JsonValue, client: Arc<FanboxClientInternal>) -> Option<Self> {
        let oitems = &value["items"];
        if !oitems.is_array() {
            return None;
        }
        let mut items = Vec::new();
        for item in oitems.members() {
            items.push(FanboxComment::new(item));
        }
        let next_url = match value["nextUrl"].as_str() {
            Some(next_url) => Some(next_url.to_owned()),
            None => None,
        };
        Some(Self {
            items,
            next_url,
            client,
        })
    }
}

impl Debug for FanboxCommentList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxCommentList")
            .field("items", &self.items)
            .field("next_url", &self.next_url)
            .finish_non_exhaustive()
    }
}
