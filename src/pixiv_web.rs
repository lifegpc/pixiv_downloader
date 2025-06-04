use crate::ext::atomic::AtomicQuick;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
use json::JsonValue;
use wreq::Response;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

/// A client which use Pixiv's web API
pub struct PixivWebClient {
    client: WebClient,
    /// true if in is initialized
    inited: AtomicBool,
    /// pixiv global data
    data: RwLock<Option<JsonValue>>,
    /// Get basic params
    params: RwLock<Option<JsonValue>>,
    login_checked: AtomicBool,
}

impl PixivWebClient {
    pub fn new() -> Self {
        Self {
            client: WebClient::default(),
            inited: AtomicBool::new(false),
            data: RwLock::new(None),
            params: RwLock::new(None),
            login_checked: AtomicBool::new(false),
        }
    }

    pub fn is_inited(&self) -> bool {
        self.inited.qload()
    }

    pub fn init(&self) -> bool {
        let helper = get_helper();
        let c = helper.cookies();
        if c.is_some() {
            if !self.client.read_cookies(c.as_ref().unwrap()) {
                return false;
            }
        }
        self.client.set_header("User-Agent", &helper.user_agent());
        let l = helper.language();
        if l.is_some() {
            self.client
                .set_header("Accept-Language", l.as_ref().unwrap());
            self.params
                .get_mut()
                .replace(json::object! { "lang": l.as_ref().unwrap().replace("-", "_").as_str() });
        } else {
            self.client.set_header("Accept-Language", "ja");
            self.params
                .get_mut()
                .replace(json::object! { "lang": "ja" });
        }
        self.inited.qstore(true);
        true
    }

    pub fn auto_init(&self) {
        if !self.is_inited() {
            let r = self.init();
            if !r {
                panic!("{}", gettext("Failed to initialize pixiv web api client."));
            }
        }
    }

    pub async fn check_login(&self) -> bool {
        self.auto_init();
        self.login_checked.qstore(true);
        let r = self
            .client
            .get_with_param("https://www.pixiv.net/", self.get_params(), None)
            .await;
        if r.is_none() {
            return false;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            log::error!(target: "pixiv_web","{} {}", gettext("Failed to get main page:"), status);
            return false;
        }
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            log::error!(
                target: "pixiv_web",
                "{} {}",
                gettext("Failed to get main page:"),
                data.unwrap_err()
            );
            return false;
        }
        let data = data.unwrap();
        let mut p = MetaDataParser::default();
        if !p.parse(data.as_str()) {
            log::error!(target: "pixiv_web","{}", gettext("Failed to parse main page."));
            return false;
        }
        log::debug!(
            target: "pixiv_web",
            "{}\n{}",
            gettext("Main page's data:"),
            p.value.as_ref().unwrap().pretty(2).as_str()
        );
        self.data.get_mut().replace(p.value.unwrap());
        true
    }

    pub async fn deal_json(&self, r: Response) -> Option<JsonValue> {
        let status = r.status();
        let code = status.as_u16();
        let is_status_err = code >= 400;
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            if is_status_err {
                log::error!(target: "pixiv_web", "HTTP ERROR {}", status);
            }
            log::error!(target: "pixiv_web","{} {}", gettext("Network error:"), data.unwrap_err());
            return None;
        }
        let data = data.unwrap();
        let re = json::parse(data.as_str());
        if re.is_err() {
            if is_status_err {
                log::error!(target: "pixiv_web","HTTP ERROR {}", status);
            } else {
                log::error!(target: "pixiv_web","{} {}", gettext("Failed to parse JSON:"), re.unwrap_err());
            }
            return None;
        }
        let value = re.unwrap();
        let error = (&value["error"]).as_bool();
        if error.is_none() {
            if is_status_err {
                log::error!(target: "pixiv_web","HTTP ERROR {}", status);
            }
            log::error!(target: "pixiv_web","{}", gettext("Failed to detect error."));
            return None;
        }
        let error = error.unwrap();
        if error {
            let message = (&value["message"]).as_str();
            if is_status_err {
                log::error!(target: "pixiv_web","HTTP ERROR {}", status);
            }
            if message.is_some() {
                log::error!(target: "pixiv_web","{}", message.unwrap());
            }
            return None;
        }
        let body = &value["body"];
        if body.is_empty() || body.is_null() {
            return Some(value);
        }
        Some(body.clone())
    }

    pub async fn get_artwork_ajax(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/ajax/illust/{}", id),
                self.get_params(),
                None,
            )
            .await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("Artwork's data:"),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    pub async fn get_artwork(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/artworks/{}", id),
                self.get_params(),
                None,
            )
            .await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            log::error!(target: "pixiv_web","{} {}", gettext("Failed to get artwork page:"), status);
            return None;
        }
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            log::error!(
                target: "pixiv_web",
                "{} {}",
                gettext("Failed to get artwork page:"),
                data.unwrap_err()
            );
            return None;
        }
        let data = data.unwrap();
        let mut p = MetaDataParser::new("preload-data");
        if !p.parse(data.as_str()) {
            log::error!(target: "pixiv_web","{}", gettext("Failed to parse artwork page."));
            return None;
        }
        log::debug!(
            target: "pixiv_web",
            "{} {}",
            gettext("Artwork's data:"),
            p.value.as_ref().unwrap().pretty(2)
        );
        Some(p.value.unwrap())
    }

    pub async fn get_illust_pages(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/ajax/illust/{}/pages", id),
                self.get_params(),
                None,
            )
            .await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("Artwork's page data:"),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    pub fn get_params(&self) -> Option<JsonValue> {
        self.params.get_ref().clone()
    }

    pub async fn get_ugoira(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/ajax/illust/{}/ugoira_meta", id),
                self.get_params(),
                None,
            )
            .await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("Ugoira's data:"),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    pub async fn get_follow(&self, page: u64, r18: bool) -> Option<JsonValue> {
        self.auto_init();
        let mut params = self.get_params().unwrap_or_else(|| json::object! {});
        params["p"] = page.into();
        params["mode"] = if r18 { "r18" } else { "all" }.into();
        let r = self
            .client
            .get_with_param(
                "https://www.pixiv.net/ajax/follow_latest/illust",
                Some(params),
                None,
            )
            .await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("Follower's new illusts: "),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    /// Get user's bookmarks
    /// * `uid` - User's id
    /// * `is_hide` - Whether to get hidden bookmarks
    /// * `tag` - Tag
    /// * `offset` - Offset
    /// * `limit` - Limit. Default: `48`
    pub async fn get_user_bookmarks(
        &self,
        uid: u64,
        is_hide: bool,
        tag: Option<&str>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Option<JsonValue> {
        self.auto_init();
        let mut params = self.get_params().unwrap_or_else(|| json::object! {});
        params["rest"] = if is_hide { "hide" } else { "show" }.into();
        params["tag"] = tag.unwrap_or("").into();
        params["offset"] = offset.unwrap_or(0).into();
        params["limit"] = limit.unwrap_or(48).into();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/ajax/user/{}/illusts/bookmarks", uid),
                Some(params),
                None,
            )
            .await;
        let r = match r {
            Some(r) => r,
            None => return None,
        };
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("User's bookmarks: "),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    /// Get user's works
    /// * `uid` - User's id
    pub async fn get_user_works(&self, uid: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self
            .client
            .get_with_param(
                format!("https://www.pixiv.net/ajax/user/{}/profile/all", uid),
                self.get_params(),
                None,
            )
            .await;
        let r = match r {
            Some(r) => r,
            None => return None,
        };
        let v = self.deal_json(r).await;
        if v.is_some() {
            log::debug!(
                target: "pixiv_web",
                "{} {}",
                gettext("User's works: "),
                v.as_ref().unwrap().pretty(2)
            );
        }
        v
    }

    pub fn logined(&self) -> bool {
        let data = self.data.get_ref();
        if data.is_none() {
            return false;
        }
        let value = data.as_ref().unwrap();
        let d = &value["userData"];
        if d.is_object() {
            return true;
        }
        false
    }

    pub fn is_login_checked(&self) -> bool {
        self.login_checked.qload()
    }
}
