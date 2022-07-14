use crate::fanbox::post::FanboxPost;
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
use json::JsonValue;

pub struct FanboxData {
    pub id: PixivID,
    /// Raw data
    pub raw: JsonValue,
}

impl FanboxData {
    pub fn new<T: ToPixivID>(id: T, post: &FanboxPost) -> Option<Self> {
        match id.to_pixiv_id() {
            Some(id) => Some(Self {
                id,
                raw: post.get_json().clone(),
            }),
            None => None,
        }
    }
}
