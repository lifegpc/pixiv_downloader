use crate::downloader::pd_file::PdFile;
use crate::ext::atomic::AtomicQuick;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
use json::JsonValue;
use reqwest::IntoUrl;
use reqwest::Response;
use spin_on::spin_on;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// A client which use Pixiv's web API
pub struct PixivWebClient {
    client: WebClient,
    /// true if in is initialized
    inited: Arc<AtomicBool>,
    /// pixiv global data
    data: RwLock<Option<JsonValue>>,
    /// Get basic params
    params: RwLock<Option<JsonValue>>,
}

impl PixivWebClient {
    pub fn new() -> Self {
        Self {
            client: WebClient::default(),
            inited: Arc::new(AtomicBool::new(false)),
            data: RwLock::new(None),
            params: RwLock::new(None),
        }
    }

    async fn aget_data_as_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Option<JsonValue>> {
        loop {
            match self.data.try_write() {
                Ok(f) => { return f; }
                Err(_) => {
                    tokio::time::sleep(Duration::new(0, 1_000_000)).await;
                }
            }
        }
    }

    fn get_data_as_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Option<JsonValue>> {
        spin_on(self.aget_data_as_mut())
    }

    async fn aget_data<'a>(&'a self) -> RwLockReadGuard<'a, Option<JsonValue>> {
        loop {
            match self.data.try_read() {
                Ok(f) => { return f; }
                Err(_) => {
                    tokio::time::sleep(Duration::new(0, 1_000_000)).await;
                }
            }
        }
    }

    fn get_data<'a>(&'a self) -> RwLockReadGuard<'a, Option<JsonValue>> {
        spin_on(self.aget_data())
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
        self.client.set_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36");
        let l = helper.language();
        if l.is_some() {
            self.client.set_header("Accept-Language", l.as_ref().unwrap());
            self.params.get_mut().replace(json::object! { "lang": l.as_ref().unwrap().replace("-", "_").as_str() });
        } else {
            self.client.set_header("Accept-Language", "ja");
            self.params.get_mut().replace(json::object! { "lang": "ja" });
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
        let r = self.client.get_with_param("https://www.pixiv.net/", self.params.get_ref(), None).await;
        if r.is_none() {
            return false;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            println!("{} {}", gettext("Failed to get main page:"), status);
            return false;
        }
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            println!("{} {}", gettext("Failed to get main page:"), data.unwrap_err());
            return false;
        }
        let data = data.unwrap();
        let mut p = MetaDataParser::default();
        if !p.parse(data.as_str()) {
            println!("{}", gettext("Failed to parse main page."));
            return false;
        }
        if get_helper().verbose() {
            println!("{}\n{}", gettext("Main page's data:"), p.value.as_ref().unwrap().pretty(2).as_str());
        }
        self.get_data_as_mut().replace(p.value.unwrap());
        true
    }

    pub async fn deal_json(&self, r: Response) -> Option<JsonValue> {
        let status = r.status();
        let code = status.as_u16();
        let is_status_err = code >= 400;
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            if is_status_err {
                println!("HTTP ERROR {}", status);
            }
            println!("{} {}", gettext("Network error:"), data.unwrap_err());
            return None;
        }
        let data = data.unwrap();
        let re = json::parse(data.as_str());
        if re.is_err() {
            if is_status_err {
                println!("HTTP ERROR {}", status);
            } else {
                println!("{} {}", gettext("Failed to parse JSON:"), re.unwrap_err());
            }
            return None;
        }
        let value = re.unwrap();
        let error = (&value["error"]).as_bool();
        if error.is_none() {
            if is_status_err {
                println!("HTTP ERROR {}", status);
            }
            println!("{}", gettext("Failed to detect error."));
            return None;
        }
        let error = error.unwrap();
        if error {
            let message = (&value["message"]).as_str();
            if is_status_err {
                println!("HTTP ERROR {}", status);
            }
            if message.is_some() {
                println!("{}", message.unwrap());
            }
            return None;
        }
        let body = &value["body"];
        if body.is_empty() || body.is_null() {
            return Some(value);
        }
        Some(body.clone())
    }

    pub async fn download_image<U: IntoUrl + Clone>(&self, url: U) -> Option<Response> {
        self.auto_init();
        let r = self.client.get(url, json::object!{"referer": "https://www.pixiv.net/"}).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            println!("{} {}", gettext("Failed to download image:"), status);
            return None;
        }
        Some(r)
    }

    pub async fn adownload_image<U: IntoUrl + Clone>(&self, url: U, pdf: &Option<PdFile>) -> Option<Response> {
        self.auto_init();
        let r = self.client.get(url, json::object!{"referer": "https://www.pixiv.net/"}).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            println!("{} {}", gettext("Failed to download image:"), status);
            return None;
        }
        Some(r)
    }

    pub async fn get_artwork_ajax(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get_with_param(format!("https://www.pixiv.net/ajax/illust/{}", id), self.params.get_ref(), None).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if get_helper().verbose() && v.is_some() {
            println!("{} {}", gettext("Artwork's data:"), v.as_ref().unwrap().pretty(2));
        }
        v
    }

    pub async fn get_artwork(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get_with_param(format!("https://www.pixiv.net/artworks/{}", id), self.params.get_ref(), None).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let status = r.status();
        let code = status.as_u16();
        if code >= 400 {
            println!("{} {}", gettext("Failed to get artwork page:"), status);
            return None;
        }
        let data = r.text_with_charset("UTF-8").await;
        if data.is_err() {
            println!("{} {}", gettext("Failed to get artwork page:"), data.unwrap_err());
            return None;
        }
        let data = data.unwrap();
        let mut p = MetaDataParser::new("preload-data");
        if !p.parse(data.as_str()) {
            println!("{}", gettext("Failed to parse artwork page."));
            return None;
        }
        if get_helper().verbose() {
            println!("{} {}", gettext("Artwork's data:"), p.value.as_ref().unwrap().pretty(2));
        }
        Some(p.value.unwrap())
    }

    pub async fn get_illust_pages(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get_with_param(format!("https://www.pixiv.net/ajax/illust/{}/pages", id), self.params.get_ref(), None).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if get_helper().verbose() && v.is_some() {
            println!("{} {}", gettext("Artwork's page data:"), v.as_ref().unwrap().pretty(2));
        }
        v
    }

    pub async fn get_ugoira(&self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get_with_param(format!("https://www.pixiv.net/ajax/illust/{}/ugoira_meta", id), self.params.get_ref(), None).await;
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r).await;
        if get_helper().verbose() && v.is_some() {
            println!("{} {}", gettext("Ugoira's data:"), v.as_ref().unwrap().pretty(2));
        }
        v
    }

    pub fn logined(&self) -> bool {
        let data = self.get_data();
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
}

impl Drop for PixivWebClient {
    fn drop(&mut self) {
        if self.is_inited() {
            let c = get_helper().cookies();
            if c.is_some() {
                if !self.client.save_cookies(c.as_ref().unwrap()) {
                    println!("{} {}", gettext("Warning: Failed to save cookies file:"), c.as_ref().unwrap());
                }
            }
        }
    }
}
