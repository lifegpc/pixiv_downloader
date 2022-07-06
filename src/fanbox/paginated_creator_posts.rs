use super::error::FanboxAPIError;
use super::item_list::FanboxItemList;
use crate::fanbox_api::FanboxClientInternal;
use crate::gettext;
use json::JsonValue;
use proc_macros::fanbox_api_test;
use std::fmt::Debug;
use std::sync::Arc;

/// List creator posts
pub struct PaginatedCreatorPosts {
    /// Fanbox API Client
    client: Arc<FanboxClientInternal>,
    /// Pages
    pages: Vec<String>,
}

impl PaginatedCreatorPosts {
    /// Create a new instance.
    /// * `value` - Data from fanbox api
    /// * `client` - The instance of the fanbox api client
    pub fn new(
        value: &JsonValue,
        client: Arc<FanboxClientInternal>,
    ) -> Result<Self, FanboxAPIError> {
        let body = &value["body"];
        if !body.is_array() {
            return Err(FanboxAPIError::from(gettext(
                "Failed to parse paginated data.",
            )));
        }
        let mut pages = Vec::new();
        for i in body.members() {
            match i.as_str() {
                Some(s) => {
                    pages.push(s.to_owned());
                }
                None => {
                    return Err(FanboxAPIError::from(gettext(
                        "Failed to parse paginated data.",
                    )));
                }
            }
        }
        Ok(Self { client, pages })
    }

    #[allow(dead_code)]
    /// Get posts' data in specified page.
    /// * `index` - The index of the page
    pub async fn get_page(&self, index: usize) -> Option<FanboxItemList> {
        if index >= self.pages.len() {
            None
        } else {
            match self
                .client
                .get_url(
                    self.pages[index].as_str(),
                    gettext("Failed to get posts' data:"),
                    gettext("Posts' data:"),
                )
                .await
            {
                Some(d) => match FanboxItemList::new(&d["body"], Arc::clone(&self.client)) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        println!("{} {}", gettext("Failed to parse posts's data:"), e);
                        None
                    }
                },
                None => None,
            }
        }
    }

    #[allow(dead_code)]
    #[inline]
    /// Returns the total pages
    pub fn len(&self) -> usize {
        self.pages.len()
    }
}

impl Debug for PaginatedCreatorPosts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.pages.len();
        let mut index = 0usize;
        if size == 0 {
            f.write_str("PaginatedCreatorPosts { Empty }")
        } else {
            f.write_str("PaginatedCreatorPosts {\n")?;
            for page in self.pages.iter() {
                f.write_fmt(format_args!("{} {}\n", index, page))?;
                index += 1;
            }
            f.write_str("}")
        }
    }
}

fanbox_api_test!(test_paginate_creator_posts, {
    match client.paginate_creator_post("mozukun43").await {
        Some(pages) => {
            println!("{:?}", pages);
            if pages.len() > 0 {
                match pages.get_page(0).await {
                    Some(data) => {
                        println!("{:?}", data);
                        if data.has_next_page() {
                            match data.get_next_page().await {
                                Ok(data) => {
                                    println!("{:?}", data);
                                }
                                Err(e) => {
                                    println!("{}", e);
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        None => {
            panic!("Failed to paginate creator's posts.")
        }
    }
});
