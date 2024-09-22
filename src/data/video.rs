#[cfg(feature = "avdict")]
use crate::avdict::AVDict;
#[cfg(feature = "avdict")]
use crate::avdict::AVDictError;
use crate::data::data::PixivData;
use crate::parser::description::parse_description;
use std::collections::HashMap;

#[cfg(feature = "avdict")]
pub fn get_video_metadata(data: &PixivData) -> Result<AVDict, AVDictError> {
    let mut d = AVDict::new();
    if data.title.is_some() {
        let t = data.title.as_ref().unwrap();
        d.set("title", t, None)?;
    }
    if data.author.is_some() {
        let au = data.author.as_ref().unwrap();
        d.set("artist", au, None)?;
    }
    if data.description.is_some() {
        let odesc = data.description.as_ref().unwrap();
        let desc = parse_description(odesc);
        let des = match &desc {
            Some(d) => d,
            None => odesc,
        };
        d.set("comment", des, None)?;
    }
    Ok(d)
}

pub fn get_video_metas(data: &PixivData) -> HashMap<String, String> {
    let mut m = HashMap::new();
    match &data.title {
        Some(t) => {
            m.insert(String::from("title"), t.clone());
        }
        None => {}
    }
    match &data.author {
        Some(a) => {
            m.insert(String::from("artist"), a.clone());
        }
        None => {}
    }
    match &data.description {
        Some(desc) => {
            let des = match parse_description(desc) {
                Some(desc) => desc,
                None => desc.to_owned(),
            };
            m.insert(String::from("comment"), des);
        }
        None => {}
    }
    m
}
