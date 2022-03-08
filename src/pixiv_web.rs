use crate::gettext;
use crate::opthelper::OptHelper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
use crate::Main;
use json::JsonValue;
use reqwest::IntoUrl;
use reqwest::Response;
use spin_on::spin_on;

/// A client which use Pixiv's web API
pub struct PixivWebClient<'a> {
    client: WebClient,
    pub helper: OptHelper<'a>,
    /// true if in is initialized
    inited: bool,
    data: Option<JsonValue>,
}

impl<'a> PixivWebClient<'a> {
    pub fn new(m: &'a Main) -> Self {
        Self {
            client: WebClient::new(),
            helper: OptHelper::new(m.cmd.as_ref().unwrap(), m.settings.as_ref().unwrap()),
            inited: false,
            data: None,
        }
    }

    pub fn init(&mut self) -> bool {
        let c = self.helper.cookies();
        if c.is_some() {
            if !self.client.read_cookies(c.as_ref().unwrap()) {
                return false;
            }
        }
        self.client.set_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36");
        let l = self.helper.language();
        if l.is_some() {
            self.client.set_header("Accept-Language", l.as_ref().unwrap());
        } else {
            self.client.set_header("Accept-Language", "ja");
        }
        self.client.verbose = self.helper.verbose();
        let retry = self.helper.retry();
        if retry.is_some() {
            self.client.retry = retry.unwrap();
        }
        self.inited = true;
        true
    }

    pub fn auto_init(&mut self) {
        if !self.inited {
            let r = self.init();
            if !r {
                panic!("{}", gettext("Failed to initialize pixiv web api client."));
            }
        }
    }

    pub fn check_login(&mut self) -> bool {
        self.auto_init();
        let r = self.client.get("https://www.pixiv.net/", None);
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
        let data = spin_on(r.text_with_charset("UTF-8"));
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
        if self.helper.verbose() {
            println!("{}\n{}", gettext("Main page's data:"), p.value.as_ref().unwrap().pretty(2).as_str());
        }
        self.data = Some(p.value.unwrap());
        true
    }

    pub fn deal_json(&mut self, r: Response) -> Option<JsonValue> {
        let status = r.status();
        let code = status.as_u16();
        let is_status_err = code >= 400;
        let data = spin_on(r.text_with_charset("UTF-8"));
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

    pub fn download_image<U: IntoUrl + Clone>(&mut self, url: U) -> Option<Response> {
        self.auto_init();
        let r = self.client.get(url, json::object!{"referer": "https://www.pixiv.net/"});
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

    pub fn get_artwork(&mut self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get(format!("https://www.pixiv.net/artworks/{}", id), None);
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
        let data = spin_on(r.text_with_charset("UTF-8"));
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
        if self.helper.verbose() {
            println!("{} {}", gettext("Artwork's data:"), p.value.as_ref().unwrap().pretty(2));
        }
        Some(p.value.unwrap())
    }

    pub fn get_illust_pages(&mut self, id: u64) -> Option<JsonValue> {
        self.auto_init();
        let r = self.client.get(format!("https://www.pixiv.net/ajax/illust/{}/pages", id), None);
        if r.is_none() {
            return None;
        }
        let r = r.unwrap();
        let v = self.deal_json(r);
        if self.helper.verbose() {
            println!("{} {}", gettext("Artwork's page data:"), v.as_ref().unwrap().pretty(2));
        }
        v
    }

    pub fn logined(&self) -> bool {
        if self.data.is_none() {
            return false;
        }
        let value = self.data.as_ref().unwrap();
        let d = &value["userData"];
        if d.is_object() {
            return true;
        }
        false
    }
}

impl <'a> Drop for PixivWebClient<'a> {
    fn drop(&mut self) {
        if self.inited {
            let c = self.helper.cookies();
            if c.is_some() {
                if !self.client.save_cookies(c.as_ref().unwrap()) {
                    println!("{} {}", gettext("Warning: Failed to save cookies file:"), c.as_ref().unwrap());
                }
            }
        }
    }
}
