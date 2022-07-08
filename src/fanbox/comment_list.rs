use super::comment::FanboxComment;
#[cfg(test)]
use super::post::FanboxPost;
use crate::fanbox_api::FanboxClientInternal;
use crate::gettext;
use json::JsonValue;
use proc_macros::fanbox_api_test;
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
    #[allow(dead_code)]
    /// Get next page.
    /// # Note
    /// If no next page presented, will return a error.
    pub async fn get_next_page(&self) -> Option<Self> {
        match &self.next_url {
            Some(next_url) => {
                match self
                    .client
                    .get_url(
                        next_url,
                        gettext("Failed to get next page of comments:"),
                        gettext("Comment's data:"),
                    )
                    .await
                {
                    Some(s) => Self::new(&s["body"], Arc::clone(&self.client)),
                    None => None,
                }
            }
            None => None,
        }
    }

    #[allow(dead_code)]
    /// Returns true if next page is presented.
    pub fn has_next_page(&self) -> bool {
        self.next_url.is_some()
    }

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

fanbox_api_test!(test_comment_list, {
    match client.get_post_info(3795935).await {
        Some(data) => {
            assert_eq!(data.id(), Some(3795935));
            assert_eq!(
                data.title(),
                Some("171🍡新刊表紙の限定差分&skebイラストラフ※R18")
            );
            match data {
                FanboxPost::Article(a) => {
                    println!("{:?}", a);
                    let list = a.comment_list().unwrap();
                    if list.has_next_page() {
                        match list.get_next_page().await {
                            Some(list) => {
                                println!("{:?}", list);
                            }
                            None => {
                                panic!("Failed to get the next page.")
                            }
                        }
                    }
                }
                FanboxPost::Unknown(data) => {
                    println!("{}", data.data);
                }
                _ => {
                    println!("{:?}", data);
                }
            }
        }
        None => {
            panic!("Failed to get the post info(171🍡新刊表紙の限定差分&skebイラストラフ※R18)");
        }
    }
});
