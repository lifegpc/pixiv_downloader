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
    #[allow(dead_code)]
    /// Get next page.
    /// # Note
    /// If no next page presented, will return a error.
    pub async fn get_next_page(&self) -> Result<FanboxItemList, FanboxAPIError> {
        match &self.next_url {
            Some(url) => {
                match self.client.get_url(url, gettext("Failed to get next page of the items."), gettext("Items data:")).await {
                    Some(data) => {
                        Self::new(&data["body"], Arc::clone(&self.client))
                    }
                    None => {
                        Err(FanboxAPIError::from("Failed to get next page."))
                    }
                }
            }
            None => {
                Err(FanboxAPIError::from("No next url, can not get next page."))
            }
        }
    }

    #[allow(dead_code)]
    /// Returns true if next page is presented.
    pub fn has_next_page(&self) -> bool {
        self.next_url.is_some()
    }

    /// Create a new instance
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
