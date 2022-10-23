#[cfg(feature = "exif")]
use super::exif::ExifDataSource;
use crate::gettext;
use crate::opt::author_name_filter::AuthorFiler;
use crate::opthelper::get_helper;
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
use json::JsonValue;
use std::convert::TryInto;
use xml::unescape;

/// Pixiv's basic data
pub struct PixivData {
    /// ID
    pub id: PixivID,
    /// The title
    pub title: Option<String>,
    /// The author
    pub author: Option<String>,
    pub description: Option<String>,
    /// Tags (Original, translated)
    pub tags: Option<Vec<(String, Option<String>)>>,
}

impl PixivData {
    pub fn new<T: ToPixivID>(id: T) -> Option<Self> {
        let i = id.to_pixiv_id();
        if i.is_none() {
            return None;
        }
        Some(Self {
            id: i.unwrap(),
            title: None,
            author: None,
            description: None,
            tags: None,
        })
    }

    /// Read data from JSON object.
    /// The object is from `https://www.pixiv.net/artworks/<id>`
    /// * `value` - The JSON object
    /// * `allow_overwrite` - Allow overwrite the data existing.
    pub fn from_web_page_data(&mut self, value: &JsonValue, allow_overwrite: bool) {
        let id: u64 = (&self.id).try_into().unwrap();
        let ids = format!("{}", id);
        self.from_web_page_ajax_data(&value["illust"][ids.as_str()], allow_overwrite)
    }

    /// Read data from JSON object.
    /// The object is from `https://www.pixiv.net/illust/ajax/<id>`
    /// * `value` - The JSON object
    /// * `allow_overwrite` - Allow overwrite the data existing.
    pub fn from_web_page_ajax_data(&mut self, value: &JsonValue, allow_overwrite: bool) {
        if self.title.is_none() || allow_overwrite {
            let title = value["illustTitle"].as_str();
            if title.is_some() {
                self.title = Some(String::from(title.unwrap()));
            }
        }
        if self.author.is_none() || allow_overwrite {
            let author = value["userName"].as_str();
            if author.is_some() {
                let au = author.unwrap();
                match get_helper().author_name_filters() {
                    Some(l) => self.author = Some(l.filter(au)),
                    None => {
                        self.author = Some(String::from(author.unwrap()));
                    }
                }
            }
        }
        if self.description.is_none() || allow_overwrite {
            let mut description = value["description"].as_str();
            if description.is_none() {
                description = value["illustComment"].as_str();
            }
            if description.is_some() {
                let re = unescape(description.unwrap());
                match re {
                    Ok(s) => {
                        self.description = Some(s);
                    }
                    Err(s) => {
                        println!("{} {}", gettext("Failed to unescape string:"), s.as_str());
                    }
                }
            }
        }
        let mut tags = Vec::new();
        for tag in value["tags"]["tags"].members() {
            if let Some(ori) = tag["tag"].as_str() {
                tags.push((
                    ori.to_owned(),
                    match tag["translation"]["en"].as_str() {
                        Some(s) => Some(s.to_owned()),
                        None => None,
                    },
                ));
            }
        }
        self.tags.replace(tags);
    }
}

#[cfg(feature = "exif")]
impl ExifDataSource for PixivData {
    fn image_author(&self) -> Option<String> {
        self.author.clone()
    }

    fn image_comment(&self) -> Option<String> {
        self.description.clone()
    }

    fn image_id(&self) -> Option<String> {
        Some(self.id.to_link())
    }

    fn image_title(&self) -> Option<String> {
        self.title.clone()
    }
}
