use crate::data::json::ToJson;
use json::JsonValue;
use regex::Regex;

lazy_static! {
    #[doc(hidden)]
    static ref RE: Regex = Regex::new("^(https?://)?(www\\.)?pixiv\\.net/artworks/(?P<id>\\d+)").unwrap();
}

/// Repesent an Pixiv ID
#[derive(Clone, Debug)]
pub enum PixivID {
    /// Artwork Id, include illust and manga
    Artwork(u64),
}

pub trait ToPixivID {
    fn to_pixiv_id(&self) -> Option<PixivID>;
}

impl PixivID {
    pub fn parse(s: &str) -> Option<PixivID> {
        let p = s.trim();
        let num = p.parse::<u64>();
        if num.is_ok() {
            return Some(PixivID::Artwork(num.unwrap()));
        }
        let re = RE.captures(s);
        if re.is_some() {
            let r = re.unwrap().name("id");
            if r.is_some() {
                let r = r.unwrap().as_str();
                let num = r.parse::<u64>();
                return Some(PixivID::Artwork(num.unwrap()));
            }
        }
        None
    }
}

impl ToJson for PixivID {
    fn to_json(&self) -> Option<JsonValue> {
        match *self {
            PixivID::Artwork(id) => {
                Some(json::value!({"type": "artwork", "id": id}))
            }
        }
    }
}

impl ToPixivID for &PixivID {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        Some((*self).clone())
    }
}

impl ToPixivID for &str {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        PixivID::parse(*self)
    }
}

impl ToPixivID for String {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        PixivID::parse(self)
    }
}

impl ToPixivID for u64 {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        Some(PixivID::Artwork(*self))
    }
}
