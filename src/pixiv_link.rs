use crate::ext::json::ToJson;
use json::JsonValue;
use regex::Regex;
use reqwest::IntoUrl;
use std::convert::TryInto;

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

    pub fn to_link(&self) -> String {
        match self {
            Self::Artwork(id) => {
                format!("https://www.pixiv.net/artworks/{}", id)
            }
        }
    }
}

impl ToJson for PixivID {
    fn to_json(&self) -> Option<JsonValue> {
        match *self {
            PixivID::Artwork(id) => {
                Some(json::value!({"type": "artwork", "id": id, "link": self.to_link()}))
            }
        }
    }
}

impl ToPixivID for PixivID {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        Some(self.clone())
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

impl<T: ToPixivID> ToPixivID for &T {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        (*self).to_pixiv_id()
    }
}

impl TryInto<u64> for PixivID {
    type Error = ();
    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            Self::Artwork(id) => { Ok(id) }
        }
    }
}

impl TryInto<u64> for &PixivID {
    type Error = ();
    fn try_into(self) -> Result<u64, Self::Error> {
        match *self {
            PixivID::Artwork(id) => { Ok(id) }
        }
    }
}

pub fn remove_track<U: IntoUrl>(url: U) -> String {
    let s = String::from(url.as_str());
    let u = urlparse::urlparse(s.as_str());
    let path = u.path.as_str();
    if path.ends_with("/jump.php") {
        if u.query.is_some() {
            let q = u.query.as_ref().unwrap();
            let re = urlparse::unquote(q);
            if re.is_ok() {
                return re.unwrap();
            }
        }
    }
    s
}
