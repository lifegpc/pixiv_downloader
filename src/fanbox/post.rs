use super::article::body::FanboxArticleBody;
use super::article::image::FanboxArticleImage;
use super::check::CheckUnknown;
use super::comment_list::FanboxCommentList;
use super::error::FanboxAPIError;
#[cfg(feature = "exif")]
use crate::data::exif::ExifDataSource;
use crate::fanbox_api::FanboxClientInternal;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::check_json_keys;
use proc_macros::fanbox_api_test;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;

/// Fanbox post's article
pub struct FanboxPostArticle {
    /// Raw data
    pub data: JsonValue,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostArticle {
    #[inline]
    pub fn body(&self) -> FanboxArticleBody {
        FanboxArticleBody::new(&self.data["body"], Arc::clone(&self.client))
    }

    #[inline]
    pub fn comment_count(&self) -> Option<u64> {
        self.data["commentCount"].as_u64()
    }

    #[inline]
    pub fn comment_list(&self) -> Option<FanboxCommentList> {
        FanboxCommentList::new(&self.data["commentList"], Arc::clone(&self.client))
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn excerpt(&self) -> Option<&str> {
        self.data["excerpt"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.data["feeRequired"].as_u64()
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.data["hasAdultContent"].as_bool()
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn image_for_share(&self) -> Option<&str> {
        self.data["imageForShare"].as_str()
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.data["isLiked"].as_bool()
    }

    #[inline]
    pub fn is_restricted(&self) -> Option<bool> {
        self.data["isRestricted"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.data["likeCount"].as_u64()
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
    pub fn next_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["nextPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn prev_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["prevPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn published_datetime(&self) -> Option<&str> {
        self.data["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn tags(&self) -> Option<Vec<&str>> {
        let mut list = Vec::new();
        let tags = &self.data["tags"];
        if tags.is_array() {
            for i in tags.members() {
                match i.as_str() {
                    Some(tag) => {
                        list.push(tag);
                    }
                    None => {
                        return None;
                    }
                }
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    pub fn updated_datetime(&self) -> Option<&str> {
        self.data["updatedDatetime"].as_str()
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

impl CheckUnknown for FanboxPostArticle {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "body",
            "id"+,
            "commentCount"+,
            "commentList",
            "coverImageUrl",
            "creatorId"+,
            "excerpt"+,
            "feeRequired"+,
            "hasAdultContent"+,
            "imageForShare",
            "isLiked"+,
            "isRestricted"+,
            "likeCount"+,
            "nextPost",
            "prevPost",
            "publishedDatetime"+,
            "restrictedFor",
            "tags"+,
            "type",
            "title"+,
            "updatedDatetime"+,
            "user": [
                "userId"+user_id,
                "iconUrl",
                "name"+,
            ],
        );
        self.body().check_unknown()?;
        match self.comment_list() {
            Some(list) => {
                for i in list.items {
                    i.check_unknown()?;
                }
            }
            None => {}
        }
        match self.next_post() {
            Some(post) => post.check_unknown()?,
            None => {}
        }
        match self.prev_post() {
            Some(post) => post.check_unknown()?,
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxPostArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostArticle")
            .field("id", &self.id())
            .field("body", &self.body())
            .field("comment_count", &self.comment_count())
            .field("comment_list", &self.comment_list())
            .field("cover_image_url", &self.cover_image_url())
            .field("creator_id", &self.creator_id())
            .field("excerpt", &self.excerpt())
            .field("fee_required", &self.fee_required())
            .field("has_adult_content", &self.has_adult_content())
            .field("image_for_share", &self.image_for_share())
            .field("is_liked", &self.is_liked())
            .field("is_restricted", &self.is_restricted())
            .field("like_count", &self.like_count())
            .field("next_post", &self.next_post())
            .field("prev_post", &self.prev_post())
            .field("published_datetime", &self.published_datetime())
            .field("tags", &self.tags())
            .field("title", &self.title())
            .field("updated_datetime", &self.updated_datetime())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}

pub struct FanboxImageBody {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxImageBody {
    #[inline]
    pub fn images(&self) -> Option<Vec<FanboxArticleImage>> {
        let images = &self.data["images"];
        if images.is_array() {
            let mut list = Vec::new();
            for i in images.members() {
                if i.is_object() {
                    list.push(FanboxArticleImage::new(i, Arc::clone(&self.client)));
                } else {
                    return None;
                }
            }
            Some(list)
        } else {
            None
        }
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
    pub fn text(&self) -> Option<&str> {
        self.data["text"].as_str()
    }
}

impl CheckUnknown for FanboxImageBody {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "text"+,
            "images"+,
        );
        match self.images() {
            Some(imgs) => {
                for img in imgs {
                    img.check_unknown()?;
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxImageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxImageBody")
            .field("text", &self.text())
            .field("images", &self.images())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "exif")]
impl ExifDataSource for FanboxImageBody {
    fn image_comment(&self) -> Option<String> {
        match self.text() {
            Some(text) => Some(text.to_owned()),
            None => None,
        }
    }
}

/// Fanbox image post
pub struct FanboxPostImage {
    /// Raw data
    pub data: JsonValue,
    /// Fanbox api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostImage {
    #[inline]
    pub fn body(&self) -> Option<FanboxImageBody> {
        let body = &self.data["body"];
        if body.is_object() {
            Some(FanboxImageBody::new(body, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn comment_count(&self) -> Option<u64> {
        self.data["commentCount"].as_u64()
    }

    #[inline]
    pub fn comment_list(&self) -> Option<FanboxCommentList> {
        FanboxCommentList::new(&self.data["commentList"], Arc::clone(&self.client))
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn excerpt(&self) -> Option<&str> {
        self.data["excerpt"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.data["feeRequired"].as_u64()
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.data["hasAdultContent"].as_bool()
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn image_for_share(&self) -> Option<&str> {
        self.data["imageForShare"].as_str()
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.data["isLiked"].as_bool()
    }

    #[inline]
    pub fn is_restricted(&self) -> Option<bool> {
        self.data["isRestricted"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.data["likeCount"].as_u64()
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
    pub fn next_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["nextPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn prev_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["prevPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn published_datetime(&self) -> Option<&str> {
        self.data["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn tags(&self) -> Option<Vec<&str>> {
        let mut list = Vec::new();
        let tags = &self.data["tags"];
        if tags.is_array() {
            for i in tags.members() {
                match i.as_str() {
                    Some(tag) => {
                        list.push(tag);
                    }
                    None => {
                        return None;
                    }
                }
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    pub fn updated_datetime(&self) -> Option<&str> {
        self.data["updatedDatetime"].as_str()
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

impl CheckUnknown for FanboxPostImage {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "body"+,
            "commentCount"+,
            "commentList",
            "coverImageUrl",
            "creatorId"+,
            "excerpt"+,
            "feeRequired"+,
            "hasAdultContent"+,
            "imageForShare",
            "isLiked"+,
            "isRestricted"+,
            "likeCount"+,
            "nextPost",
            "prevPost",
            "publishedDatetime"+,
            "restrictedFor",
            "tags"+,
            "title"+,
            "type",
            "updatedDatetime"+,
            "user": [
                "userId"+user_id,
                "iconUrl",
                "name"+,
            ],
        );
        match self.body() {
            Some(body) => {
                body.check_unknown()?;
            }
            None => {}
        }
        match self.comment_list() {
            Some(list) => {
                for i in list.items {
                    i.check_unknown()?;
                }
            }
            None => {}
        }
        match self.next_post() {
            Some(post) => post.check_unknown()?,
            None => {}
        }
        match self.prev_post() {
            Some(post) => post.check_unknown()?,
            None => {}
        }
        Ok(())
    }
}

impl Debug for FanboxPostImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostImage")
            .field("id", &self.id())
            .field("body", &self.body())
            .field("comment_count", &self.comment_count())
            .field("comment_list", &self.comment_list())
            .field("cover_image_url", &self.cover_image_url())
            .field("creator_id", &self.creator_id())
            .field("excerpt", &self.excerpt())
            .field("fee_required", &self.fee_required())
            .field("has_adult_content", &self.has_adult_content())
            .field("image_for_share", &self.image_for_share())
            .field("is_liked", &self.is_liked())
            .field("is_restricted", &self.is_restricted())
            .field("like_count", &self.like_count())
            .field("next_post", &self.next_post())
            .field("prev_post", &self.prev_post())
            .field("published_datetime", &self.published_datetime())
            .field("tags", &self.tags())
            .field("title", &self.title())
            .field("updated_datetime", &self.updated_datetime())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "exif")]
impl ExifDataSource for FanboxPostImage {
    fn image_author(&self) -> Option<String> {
        match self.user_name() {
            Some(u) => Some(u.to_owned()),
            None => None,
        }
    }

    fn image_comment(&self) -> Option<String> {
        match self.body() {
            Some(body) => body.image_comment(),
            None => None,
        }
    }

    fn image_title(&self) -> Option<String> {
        match self.title() {
            Some(t) => Some(t.to_owned()),
            None => None,
        }
    }
}

/// A reference to another post
pub struct FanboxPostRef {
    /// Raw data
    pub data: JsonValue,
    /// The api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostRef {
    #[allow(dead_code)]
    pub async fn get_post(&self) -> Option<FanboxPost> {
        match self.id() {
            Some(id) => match self.client.get_post_info(id).await {
                Some(s) => Some(FanboxPost::new(&s["body"], Arc::clone(&self.client))),
                None => None,
            },
            None => None,
        }
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
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
    pub fn published_datetime(&self) -> Option<&str> {
        self.data["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }
}

impl CheckUnknown for FanboxPostRef {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "publishedDatetime"+,
            "title"+,
        );
        Ok(())
    }
}

impl Debug for FanboxPostRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPostRef")
            .field("id", &self.id())
            .field("published_datetime", &self.published_datetime())
            .field("title", &self.title())
            .finish_non_exhaustive()
    }
}

/// Unknown type
pub struct FanboxPostUnknown {
    /// Raw data
    pub data: JsonValue,
    /// The api client
    client: Arc<FanboxClientInternal>,
}

impl FanboxPostUnknown {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }
}

#[allow(dead_code)]
impl FanboxPostUnknown {
    #[inline]
    pub fn comment_list(&self) -> Option<FanboxCommentList> {
        FanboxCommentList::new(&self.data["commentList"], Arc::clone(&self.client))
    }

    #[inline]
    pub fn next_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["nextPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }

    #[inline]
    pub fn prev_post(&self) -> Option<FanboxPostRef> {
        let obj = &self.data["prevPost"];
        if obj.is_object() {
            Some(FanboxPostRef::new(obj, Arc::clone(&self.client)))
        } else {
            None
        }
    }
}

impl Debug for FanboxPostUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.data, f)
    }
}

impl Display for FanboxPostUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.data, f)
    }
}

#[derive(proc_macros::CheckUnknown, Debug)]
/// The fanbox's post
pub enum FanboxPost {
    /// Article
    Article(FanboxPostArticle),
    /// Image
    Image(FanboxPostImage),
    /// Unknown
    Unknown(FanboxPostUnknown),
}

impl FanboxPost {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        match data["type"].as_str() {
            Some(s) => match s {
                "article" => Self::Article(FanboxPostArticle::new(data, client)),
                "image" => Self::Image(FanboxPostImage::new(data, client)),
                _ => Self::Unknown(FanboxPostUnknown::new(data, client)),
            },
            _ => Self::Unknown(FanboxPostUnknown::new(data, client)),
        }
    }
}

#[allow(dead_code)]
impl FanboxPost {
    #[inline]
    pub fn comment_count(&self) -> Option<u64> {
        self.get_json()["commentCount"].as_u64()
    }

    #[inline]
    pub fn comment_list(&self) -> Option<FanboxCommentList> {
        match self {
            Self::Article(a) => a.comment_list(),
            Self::Image(i) => i.comment_list(),
            Self::Unknown(u) => u.comment_list(),
        }
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.get_json()["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.get_json()["creatorId"].as_str()
    }

    #[inline]
    pub fn excerpt(&self) -> Option<&str> {
        self.get_json()["excerpt"].as_str()
    }

    #[inline]
    pub fn fee_required(&self) -> Option<u64> {
        self.get_json()["feeRequired"].as_u64()
    }

    #[inline]
    pub fn get_json(&self) -> &JsonValue {
        match self {
            Self::Article(a) => &a.data,
            Self::Image(a) => &a.data,
            Self::Unknown(a) => &a.data,
        }
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.get_json()["hasAdultContent"].as_bool()
    }

    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.get_json()["id"])
    }

    #[inline]
    pub fn image_for_share(&self) -> Option<&str> {
        self.get_json()["imageForShare"].as_str()
    }

    #[inline]
    pub fn is_liked(&self) -> Option<bool> {
        self.get_json()["isLiked"].as_bool()
    }

    #[inline]
    pub fn is_restricted(&self) -> Option<bool> {
        self.get_json()["isRestricted"].as_bool()
    }

    #[inline]
    pub fn like_count(&self) -> Option<u64> {
        self.get_json()["likeCount"].as_u64()
    }

    #[inline]
    pub fn next_post(&self) -> Option<FanboxPostRef> {
        match self {
            Self::Article(a) => a.next_post(),
            Self::Image(i) => i.next_post(),
            Self::Unknown(u) => u.next_post(),
        }
    }

    #[inline]
    pub fn prev_post(&self) -> Option<FanboxPostRef> {
        match self {
            Self::Article(a) => a.prev_post(),
            Self::Image(i) => i.prev_post(),
            Self::Unknown(u) => u.prev_post(),
        }
    }

    #[inline]
    pub fn published_datetime(&self) -> Option<&str> {
        self.get_json()["publishedDatetime"].as_str()
    }

    #[inline]
    pub fn tags(&self) -> Option<Vec<&str>> {
        let mut list = Vec::new();
        let tags = &self.get_json()["tags"];
        if tags.is_array() {
            for i in tags.members() {
                match i.as_str() {
                    Some(tag) => {
                        list.push(tag);
                    }
                    None => {
                        return None;
                    }
                }
            }
            Some(list)
        } else {
            None
        }
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.get_json()["title"].as_str()
    }

    #[inline]
    pub fn updated_datetime(&self) -> Option<&str> {
        self.get_json()["updatedDatetime"].as_str()
    }

    #[inline]
    pub fn user_icon_url(&self) -> Option<&str> {
        self.get_json()["user"]["iconUrl"].as_str()
    }

    #[inline]
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.get_json()["user"]["userId"])
    }

    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.get_json()["user"]["name"].as_str()
    }
}

fanbox_api_test!(test_get_post_info, {
    match client.get_post_info(4070200).await {
        Some(data) => {
            assert_eq!(data.id(), Some(4070200));
            match data {
                FanboxPost::Article(data) => {
                    println!("{:#?}", data);
                    assert_eq!(data.user_id(), Some(705370));
                    assert_eq!(
                        data.title(),
                        Some("第253話 【壁紙プレゼント】ひんやりレモンころねちゃん🍋")
                    );
                    assert_eq!(data.user_name(), Some("しらたま"));
                    assert_eq!(data.creator_id(), Some("shiratamaco"));
                    match data.check_unknown() {
                        Ok(_) => {}
                        Err(e) => {
                            panic!("Check unknown: {}", e);
                        }
                    }
                    match data.next_post() {
                        Some(r) => {
                            println!("{:#?}", r);
                            match r.get_post().await {
                                Some(n) => {
                                    println!("{:#?}", n);
                                    assert_eq!(n.id(), r.id());
                                    assert_eq!(n.user_id(), Some(705370));
                                    assert_eq!(n.user_name(), Some("しらたま"));
                                    assert_eq!(n.creator_id(), Some("shiratamaco"));
                                    match n.check_unknown() {
                                        Ok(_) => {}
                                        Err(e) => {
                                            panic!("Check unknown: {}", e);
                                        }
                                    }
                                }
                                None => {
                                    panic!("Failed to get next post.")
                                }
                            }
                        }
                        None => {}
                    }
                }
                FanboxPost::Image(data) => {
                    println!("{:#?}", data);
                }
                FanboxPost::Unknown(data) => {
                    println!("{}", data);
                }
            }
        }
        None => {
            panic!("Failed to get the post info(第253話 【壁紙プレゼント】ひんやりレモンころねちゃん🍋).");
        }
    }
});

fanbox_api_test!(test_get_post_info2, {
    match client.get_post_info(3972093).await {
        Some(data) => {
            assert_eq!(data.id(), Some(3972093));
            match &data {
                FanboxPost::Image(img) => {
                    println!("{:#?}", img);
                }
                FanboxPost::Unknown(data) => {
                    println!("{}", data.data);
                }
                _ => {
                    println!("{:#?}", data);
                }
            }
            match data.check_unknown() {
                Ok(_) => {}
                Err(e) => {
                    panic!("Check unknown: {}", e);
                }
            }
        }
        None => {
            panic!("Failed to get the post info(【高解像度JPG】お正月プレゼントはチノ?)");
        }
    }
});
