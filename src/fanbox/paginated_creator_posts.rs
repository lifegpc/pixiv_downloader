#[cfg(test)]
use crate::fanbox_api::FanboxClient;
use crate::fanbox_api::FanboxClientInternal;
use crate::gettext;
use json::JsonValue;
use proc_macros::fanbox_api_test;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum PaginatedCreatorPostsError {
    String(String),
}

impl From<&'static str> for PaginatedCreatorPostsError {
    fn from(v: &'static str) -> Self {
        Self::String(v.to_owned())
    }
}

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
    ) -> Result<Self, PaginatedCreatorPostsError> {
        let body = &value["body"];
        if !body.is_array() {
            return Err(PaginatedCreatorPostsError::from(gettext(
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
                    return Err(PaginatedCreatorPostsError::from(gettext(
                        "Failed to parse paginated data.",
                    )));
                }
            }
        }
        Ok(Self { client, pages })
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
            f.write_str("}\n")
        }
    }
}

fanbox_api_test!(test_paginate_creator_posts, {
    match client.paginate_creator_post("mozukun43").await {
        Some(pages) => {
            println!("{:?}", pages);
        }
        None => {
            panic!("Failed to paginate creator's posts.")
        }
    }
});
