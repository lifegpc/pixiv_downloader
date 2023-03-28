use crate::ext::json::ToJson;
use json::JsonValue;
use regex::Regex;
use reqwest::IntoUrl;

lazy_static! {
    #[doc(hidden)]
    static ref RE: Regex = Regex::new("^(https?://)?(www\\.)?pixiv\\.net/artworks/(?P<id>\\d+)").unwrap();
    #[doc(hidden)]
    static ref RE2: Regex = Regex::new("^(https?://)?(www\\.)?fanbox\\.cc/@(?P<creator>[^/]+)/posts/(?P<id>\\d+)").unwrap();
    #[doc(hidden)]
    static ref RE3: Regex = Regex::new("^(https?://)?(?P<creator>[^./]+)\\.fanbox\\.cc/posts/(?P<id>\\d+)").unwrap();
    #[doc(hidden)]
    static ref RE4: Regex = Regex::new("^(https?://)?(?P<creator>[^./]+)\\.fanbox\\.cc(/(\\?.*)?)?$").unwrap();
    #[doc(hidden)]
    static ref RE5: Regex = Regex::new("^(https?://)?(www\\.)?fanbox\\.cc/@(?P<creator>[^/?]+)(/(\\?.*)?)?$").unwrap();
}

#[derive(Clone, Debug)]
/// Fanbox post ID
pub struct FanboxPostID {
    /// Creator ID
    pub creator_id: String,
    /// Post ID
    pub post_id: u64,
}

impl FanboxPostID {
    /// Create a new id.
    /// * `creator_id` - Creator ID
    /// * `post_id` - Post ID
    pub fn new<C: AsRef<str> + ?Sized>(creator_id: &C, post_id: u64) -> Self {
        Self {
            creator_id: String::from(creator_id.as_ref()),
            post_id,
        }
    }
}

/// Repesent an Pixiv ID
#[derive(Clone, Debug)]
pub enum PixivID {
    /// Artwork Id, include illust and manga
    Artwork(u64),
    /// Fanbox post
    FanboxPost(FanboxPostID),
    /// Fanbox creator
    FanboxCreator(String),
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
        match RE.captures(s) {
            Some(re) => match re.name("id") {
                Some(r) => match r.as_str().parse::<u64>() {
                    Ok(r) => return Some(Self::Artwork(r)),
                    Err(_) => {}
                },
                None => {}
            },
            None => {}
        }
        match RE2.captures(s) {
            Some(re) => match re.name("creator") {
                Some(creator) => match re.name("id") {
                    Some(id) => match id.as_str().parse::<u64>() {
                        Ok(id) => {
                            return Some(Self::FanboxPost(FanboxPostID::new(creator.as_str(), id)));
                        }
                        Err(_) => {}
                    },
                    None => {}
                },
                None => {}
            },
            None => {}
        }
        match RE3.captures(s) {
            Some(re) => match re.name("creator") {
                Some(creator) => match re.name("id") {
                    Some(id) => match id.as_str().parse::<u64>() {
                        Ok(id) => {
                            return Some(Self::FanboxPost(FanboxPostID::new(creator.as_str(), id)));
                        }
                        Err(_) => {}
                    },
                    None => {}
                },
                None => {}
            },
            None => {}
        }
        match RE4.captures(s) {
            Some(re) => match re.name("creator") {
                Some(creator) => match creator.as_str() {
                    "www" => {}
                    _ => return Some(Self::FanboxCreator(String::from(creator.as_str()))),
                },
                None => {}
            },
            None => {}
        }
        match RE5.captures(s) {
            Some(re) => match re.name("creator") {
                Some(creator) => {
                    return Some(Self::FanboxCreator(String::from(creator.as_str())));
                }
                None => {}
            },
            None => {}
        }
        None
    }

    pub fn to_link(&self) -> String {
        match self {
            Self::Artwork(id) => {
                format!("https://www.pixiv.net/artworks/{}", id)
            }
            Self::FanboxPost(id) => {
                format!(
                    "https://www.fanbox.cc/@{}/posts/{}",
                    id.creator_id, id.post_id
                )
            }
            Self::FanboxCreator(id) => {
                format!("https://www.fanbox.cc/@{}", id)
            }
        }
    }
}

impl ToJson for PixivID {
    fn to_json(&self) -> Option<JsonValue> {
        match &self {
            &PixivID::Artwork(id) => {
                Some(json::value!({"type": "artwork", "id": id.clone(), "link": self.to_link()}))
            }
            &PixivID::FanboxPost(id) => Some(
                json::value!({"type": "fanbox_post", "post_id": id.post_id.clone(), "creator_id": id.creator_id.clone(), "link": self.to_link()}),
            ),
            &PixivID::FanboxCreator(id) => Some(
                json::value!({"type": "fanbox_creator", "creator_id": id.clone(), "link": self.to_link()}),
            ),
        }
    }
}

impl ToPixivID for FanboxPostID {
    fn to_pixiv_id(&self) -> Option<PixivID> {
        Some(PixivID::FanboxPost(self.clone()))
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
            Self::Artwork(id) => Ok(id),
            Self::FanboxPost(id) => Ok(id.post_id),
            Self::FanboxCreator(_) => Err(()),
        }
    }
}

impl TryInto<u64> for &PixivID {
    type Error = ();
    fn try_into(self) -> Result<u64, Self::Error> {
        match self {
            PixivID::Artwork(id) => Ok(id.clone()),
            PixivID::FanboxPost(id) => Ok(id.post_id.clone()),
            PixivID::FanboxCreator(_) => Err(()),
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
