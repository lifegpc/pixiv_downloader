use super::error::FanboxAPIError;
use super::item::FanboxItem;
use crate::fanbox_api::FanboxClientInternal;
use crate::gettext;
use json::JsonValue;
use std::fmt::Debug;
use std::sync::Arc;

/// A list of fanbox item.
pub struct FanboxItemList {
    /// Item list
    pub items: Vec<FanboxItem>,
    /// The url of the next page
    next_url: Option<String>,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxItemList {
    pub fn new(
        value: &JsonValue,
        client: Arc<FanboxClientInternal>,
    ) -> Result<Self, FanboxAPIError> {
        let oitems = &value["items"];
        if !oitems.is_array() {
            return Err(FanboxAPIError::from(gettext("Failed to parse item list.")));
        }
        let mut items = Vec::new();
        for item in oitems.members() {
            items.push(FanboxItem::from(item));
        }
        let next_url = match value["nextUrl"].as_str() {
            Some(next_url) => Some(next_url.to_owned()),
            None => None,
        };
        Ok(Self {
            items,
            next_url,
            client,
        })
    }
}

impl Debug for FanboxItemList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxItemList")
            .field("items", &self.items)
            .field("next_url", &self.next_url)
            .finish_non_exhaustive()
    }
}
