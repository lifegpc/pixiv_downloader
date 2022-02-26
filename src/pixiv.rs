use crate::gettext;
use crate::opthelper::OptHelper;
use crate::parser::mainpage::MainPageParser;
use crate::webclient::WebClient;
use crate::Main;
use spin_on::spin_on;

/// A client which use Pixiv's web API
pub struct PixivWebClient<'a> {
    client: WebClient,
    helper: OptHelper<'a>,
    /// true if in is initialized
    inited: bool,
}

impl<'a> PixivWebClient<'a> {
    pub fn new(m: &'a Main) -> Self {
        Self {
            client: WebClient::new(),
            helper: OptHelper::new(m.cmd.as_ref().unwrap(), m.settings.as_ref().unwrap()),
            inited: false,
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
        let r = self.client.get("https://www.pixiv.net/");
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
        let mut p = MainPageParser::new();
        if !p.parse(data.as_str()) {
            println!("{}", gettext("Failed to parse main page."));
            return false;
        }
        true
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
