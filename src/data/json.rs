use super::fanbox::FanboxData;
use crate::data::data::PixivData;
use crate::ext::json::{ToJson, ToJson2};
use crate::gettext;
use crate::parser::description::{convert_description_to_md, parse_description};
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
use int_enum::IntEnum;
use json::JsonValue;
use std::collections::HashMap;
use std::convert::From;
use std::ffi::OsStr;
use std::fs::remove_file;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// Store metadata informations in JSON file
pub struct JSONDataFile {
    id: PixivID,
    maps: HashMap<String, JsonValue>,
}

impl JSONDataFile {
    #[allow(dead_code)]
    pub fn new<T: ToPixivID>(id: T) -> Option<Self> {
        let i = id.to_pixiv_id();
        if i.is_none() {
            return None;
        }
        Some(Self {
            id: i.unwrap(),
            maps: HashMap::new(),
        })
    }

    pub fn add<T: ToJson2>(&mut self, key: &str, value: T) {
        let v = value.to_json2();
        self.maps.insert(String::from(key), v);
    }

    pub fn save<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> bool {
        let p = Path::new(path);
        if p.exists() {
            let r = remove_file(p);
            if r.is_err() {
                log::error!("{} {}", gettext("Failed to remove file:"), r.unwrap_err());
                return false;
            }
        }
        let value = self.to_json();
        if value.is_none() {
            return false;
        }
        let value = value.unwrap();
        let f = File::create(p);
        if f.is_err() {
            log::error!("{} {}", gettext("Failed to create file:"), f.unwrap_err());
            return false;
        }
        let mut f = f.unwrap();
        let r = f.write(value.pretty(2).as_bytes());
        if r.is_err() {
            log::error!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
            return false;
        }
        true
    }
}

impl From<PixivData> for JSONDataFile {
    fn from(p: PixivData) -> Self {
        JSONDataFile::from(&p)
    }
}

impl From<Arc<PixivData>> for JSONDataFile {
    fn from(p: Arc<PixivData>) -> Self {
        JSONDataFile::from(p.as_ref())
    }
}

impl From<&PixivData> for JSONDataFile {
    fn from(p: &PixivData) -> Self {
        let mut f = Self {
            id: p.id.clone(),
            maps: HashMap::new(),
        };
        if p.title.is_some() {
            f.add("title", p.title.as_ref().unwrap());
        }
        if p.author.is_some() {
            f.add("author", p.author.as_ref().unwrap());
        }
        if p.description.is_some() {
            let desc = p.description.as_ref().unwrap();
            f.add("description", desc);
            let pd = parse_description(desc);
            if pd.is_some() {
                f.add("parsed_description", pd.unwrap());
            }
            let md = convert_description_to_md(desc);
            if md.is_ok() {
                f.add("markdown_description", md.unwrap());
            }
        }
        match p.tags.as_ref() {
            Some(tags) => {
                let mut t = JsonValue::new_array();
                for tag in tags {
                    t.push(json::array![
                        tag.0.as_str(),
                        match &tag.1 {
                            Some(s) => JsonValue::String(s.to_owned()),
                            None => JsonValue::Null,
                        }
                    ])
                    .unwrap();
                }
                f.add("tags", t);
            }
            None => {}
        }
        match &p.ai_type {
            Some(ai_type) => {
                f.add("is_ai", ai_type.is_ai());
                f.add("ai_type", ai_type.int_value());
            }
            None => {
                f.add("is_ai", JsonValue::Null);
                f.add("ai_type", JsonValue::Null);
            }
        }
        f
    }
}

impl From<FanboxData> for JSONDataFile {
    fn from(d: FanboxData) -> Self {
        let mut f = Self {
            id: d.id.clone(),
            maps: HashMap::new(),
        };
        f.add("raw", d.raw);
        f
    }
}

impl From<&FanboxData> for JSONDataFile {
    fn from(d: &FanboxData) -> Self {
        let mut f = Self {
            id: d.id.clone(),
            maps: HashMap::new(),
        };
        f.add("raw", d.raw.clone());
        f
    }
}

impl ToJson for JSONDataFile {
    fn to_json(&self) -> Option<JsonValue> {
        let mut value = json::object! {};
        if value.insert("id", self.id.to_json()).is_err() {
            return None;
        }
        for (k, v) in self.maps.iter() {
            if value.insert(k, v.clone()).is_err() {
                return None;
            }
        }
        Some(value)
    }
}
