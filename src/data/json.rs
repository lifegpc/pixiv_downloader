use super::fanbox::FanboxData;
use crate::data::data::PixivData;
use crate::ext::json::ToJson;
use crate::gettext;
use crate::parser::description::parse_description;
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
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

    pub fn add<T: ToJson>(&mut self, key: &str, value: T) -> Result<(), ()> {
        let v = value.to_json();
        if v.is_some() {
            self.maps.insert(String::from(key), v.unwrap());
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn save<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> bool {
        let p = Path::new(path);
        if p.exists() {
            let r = remove_file(p);
            if r.is_err() {
                println!("{} {}", gettext("Failed to remove file:"), r.unwrap_err());
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
            println!("{} {}", gettext("Failed to create file:"), f.unwrap_err());
            return false;
        }
        let mut f = f.unwrap();
        let r = f.write(value.pretty(2).as_bytes());
        if r.is_err() {
            println!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
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
            f.add("title", p.title.as_ref().unwrap()).unwrap();
        }
        if p.author.is_some() {
            f.add("author", p.author.as_ref().unwrap()).unwrap();
        }
        if p.description.is_some() {
            f.add("description", p.description.as_ref().unwrap())
                .unwrap();
            let pd = parse_description(p.description.as_ref().unwrap());
            if pd.is_some() {
                f.add("parsed_description", pd.unwrap()).unwrap();
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
                f.add("tags", t).unwrap();
            }
            None => {}
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
        f.add("raw", d.raw).unwrap();
        f
    }
}

impl From<&FanboxData> for JSONDataFile {
    fn from(d: &FanboxData) -> Self {
        let mut f = Self {
            id: d.id.clone(),
            maps: HashMap::new(),
        };
        f.add("raw", d.raw.clone()).unwrap();
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
