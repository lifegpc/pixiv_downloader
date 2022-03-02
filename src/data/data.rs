use crate::gettext;
use crate::pixiv_link::ToPixivID;
use crate::pixiv_link::PixivID;
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
        })
    }

    /// Read data from JSON object.
    /// The object is from https://www.pixiv.net/artworks/<id>
    /// * `value` - The JSON object
    /// * `allow_overwrite` - Allow overwrite the data existing.
    pub fn from_web_page_data(&mut self, value: &JsonValue, allow_overwrite: bool) {
        let id: u64 = (&self.id).try_into().unwrap();
        let ids = format!("{}", id);
        if self.title.is_none() || allow_overwrite {
            let title = value["illust"][ids.as_str()]["illustTitle"].as_str();
            if title.is_some() {
                self.title = Some(String::from(title.unwrap()));
            }
        }
        if self.author.is_none() || allow_overwrite {
            let author = value["illust"][ids.as_str()]["userName"].as_str();
            if author.is_some() {
                self.author = Some(String::from(author.unwrap()));
            }
        }
        if self.description.is_none() || allow_overwrite {
            let mut description = value["illust"][ids.as_str()]["description"].as_str();
            if description.is_none() {
                description = value["illust"][ids.as_str()]["illustComment"].as_str();
            }
            if description.is_some() {
                let re = unescape(description.unwrap());
                match re {
                    Ok(s) => { self.description = Some(s); }
                    Err(s) => {
                        println!("{} {}", gettext("Failed to unescape string:"), s.as_str());
                    }
                }
            }
        }
    }
}
