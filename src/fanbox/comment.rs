use super::check::CheckUnknown;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;

pub struct FanboxComment {
    pub data: JsonValue,
}

impl FanboxComment {
    #[inline]
    pub fn body(&self) -> Option<&str> {
        self.data["body"].as_str()
    }

    #[inline]
    pub fn created_datetime(&self) -> Option<&str> {
        self.data["createdDatetime"].as_str()
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.data["isLiked"].as_bool()
    }

    #[inline]
    pub fn is_own(&self) -> Option<bool> {
        self.data["isOwn"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.data["likeCount"].as_u64()
    }

    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }

    #[inline]
    pub fn parent_comment_id(&self) -> Option<u64> {
        parse_u64(&self.data["parentCommentId"])
    }

    #[inline]
    pub fn replies(&self) -> Option<Vec<FanboxComment>> {
        let r = &self.data["replies"];
        if r.is_array() {
            let mut list = Vec::new();
            for i in r.members() {
                list.push(Self::new(i));
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn root_comment_id(&self) -> Option<u64> {
        parse_u64(&self.data["rootCommentId"])
    }

    #[inline]
    pub fn user_icon_url(&self) -> Option<&str> {
        self.data["user"]["iconUrl"].as_str()
    }

    #[inline]
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["user"]["userId"])
    }

    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.data["user"]["name"].as_str()
    }
}

impl CheckUnknown for FanboxComment {
    fn check_unknown(&self) -> Result<(), super::error::FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "body"+,
            "createdDatetime"+,
            "isLiked"+,
            "isOwn"+,
            "likeCount"+,
            "parentCommentId",
            "replies",
            "rootCommentId",
            "user": [
                "userId"+user_id,
                "name"+,
                "iconUrl",
            ],
        );
        match self.replies() {
            Some(replies) => {
                for i in replies {
                    i.check_unknown()?;
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxComment")
            .field("id", &self.id())
            .field("body", &self.body())
            .field("created_datetime", &self.created_datetime())
            .field("is_liked", &self.is_liked())
            .field("is_own", &self.is_own())
            .field("like_count", &self.like_count())
            .field("parent_comment_id", &self.parent_comment_id())
            .field("replies", &self.replies())
            .field("root_comment_id", &self.root_comment_id())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}
