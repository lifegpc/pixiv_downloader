use crate::gettext;
use crate::pixiv_link::PixivID;
use crate::pixiv_link::ToPixivID;
use json::JsonValue;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::remove_file;
use std::io::Write;
use std::path::Path;

pub trait ToJson {
    fn to_json(&self) -> Option<JsonValue>;
}

/// Store metadata informations in JSON file
pub struct JSONDataFile {
    id: PixivID,
    maps: HashMap<String, JsonValue>,
}

impl JSONDataFile {
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
