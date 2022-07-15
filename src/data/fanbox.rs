#[cfg(feature = "exif")]
use super::exif::ExifDataSource;
use crate::fanbox::post::FanboxPost;
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
use json::JsonValue;
#[cfg(feature = "exif")]
use proc_macros::call_parent_data_source_fun;

pub struct FanboxData {
    pub id: PixivID,
    /// Raw data
    pub raw: JsonValue,
    #[cfg(feature = "exif")]
    pub exif_data: Option<Box<dyn ExifDataSource>>,
}

impl FanboxData {
    pub fn new<T: ToPixivID>(id: T, post: &FanboxPost) -> Option<Self> {
        match id.to_pixiv_id() {
            Some(id) => Some(Self {
                id,
                raw: post.get_json().clone(),
                #[cfg(feature = "exif")]
                exif_data: None,
            }),
            None => None,
        }
    }
}

#[cfg(feature = "exif")]
impl ExifDataSource for FanboxData {
    call_parent_data_source_fun!(
        "src/data/exif_data_source.json",
        match &self.exif_data {
            Some(d) => d,
            None => {
                return None;
            }
        },
        image_id,
    );

    fn image_id(&self) -> Option<String> {
        Some(self.id.to_link())
    }
}
