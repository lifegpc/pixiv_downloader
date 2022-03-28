use crate::author_name_filter::AuthorFiler;
use crate::gettext;
use crate::opthelper::OptHelper;
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
    helper: OptHelper,
}

impl PixivData {
    pub fn new<T: ToPixivID>(id: T, helper: OptHelper) -> Option<Self> {
        let i = id.to_pixiv_id();
        if i.is_none() {
            return None;
        }
        Some(Self {
            id: i.unwrap(),
            title: None,
            author: None,
            description: None,
            helper: helper,
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
                match self.helper.author_name_filters() {
                    Some(l) => { self.author = Some(l.filter(au)) }
                    None => { self.author = Some(String::from(author.unwrap())); }
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
                    Ok(s) => { self.description = Some(s); }
                    Err(s) => {
                        println!("{} {}", gettext("Failed to unescape string:"), s.as_str());
                    }
                }
            }
        }
    }
}
