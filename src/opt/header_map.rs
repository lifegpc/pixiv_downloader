use crate::error::PixivDownloaderError;
use crate::ext::json::{FromJson, ToJson};
use json::JsonValue;
use std::collections::HashMap;

pub type HeaderMap = HashMap<String, String>;

impl FromJson for HeaderMap {
    type Err = PixivDownloaderError;
    fn from_json<T: ToJson>(v: T) -> Result<Self, Self::Err> {
        let j = v.to_json();
        match &j {
            Some(j) => {
                let mut map = HeaderMap::new();
                for (k, v) in j.entries() {
                    if v.is_string() {
                        map.insert(k.to_owned(), v.as_str().unwrap().to_owned());
                    } else {
                        return Err(PixivDownloaderError::from(format!(
                            "Key {} is not string.",
                            k
                        )));
                    }
                }
                Ok(map)
            }
            None => Ok(HeaderMap::new()),
        }
    }
}

pub fn check_header_map(obj: &JsonValue) -> bool {
    HeaderMap::from_json(obj).is_ok()
}
