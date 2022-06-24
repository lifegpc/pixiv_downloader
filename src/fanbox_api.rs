use crate::ext::atomic::AtomicQuick;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
use json::JsonValue;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

/// Fanbox API client
pub struct FanboxClient {
    /// Web client
    client: WebClient,
    /// true if in is initialized
    inited: AtomicBool,
    /// Fanbox global data
    data: RwLock<Option<JsonValue>>,
}

impl FanboxClient {
    /// Create an new instance
    pub fn new() -> Self {
        Self {
            client: WebClient::default(),
            inited: AtomicBool::new(false),
            data: RwLock::new(None),
        }
    }

    /// Initiailze the interface.
    ///
    /// Returns true if successed
    pub fn init(&self) -> bool {
        let helper = get_helper();
        let c = helper.cookies();
        if c.is_some() {
            if !self.client.read_cookies(c.as_ref().unwrap()) {
                return false;
            }
        }
        self.client.set_header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36");
        self.inited.qstore(true);
        true
    }

    /// Returns true if is initialized.
    pub fn is_inited(&self) -> bool {
        self.inited.qload()
    }

    /// Initialize the client if needed.
    fn auto_init(&self) {
        if !self.is_inited() {
            let r = self.init();
            if !r {
                panic!("{}", gettext("Failed to initialize pixiv web api client."));
            }
        }
    }

    /// Check login status.
    pub async fn check_login(&self) -> bool {
        self.auto_init();
        let r = self.client.get("https://www.fanbox.cc", None).await;
        match r {
            Some(r) => {
                let status = r.status();
                let code = status.as_u16();
                if code >= 400 {
                    println!("{} {}", gettext("Failed to get fanbox main page:"), status);
                    return false;
                }
                match r.text_with_charset("UTF-8").await {
                    Ok(data) => {
                        let mut parser = MetaDataParser::new("metadata");
                        if !parser.parse(data.as_str()) {
                            println!("{}", gettext("Failed to parse fanbox main page."));
                            return false;
                        }
                        if get_helper().verbose() {
                            println!(
                                "{}\n{}",
                                gettext("Fanbox main page's data:"),
                                parser.value.as_ref().unwrap().pretty(2).as_str()
                            );
                        }
                        self.data.replace_with2(parser.value);
                        true
                    }
                    Err(e) => {
                        println!("{} {}", gettext("Failed to get fanbox main page:"), e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    /// Returns true if is logged in.
    pub fn logined(&self) -> bool {
        let data = self.data.get_ref();
        if data.is_none() {
            return false;
        }
        let value = data.as_ref().unwrap();
        match value["user"]["isLoggedIn"].as_bool() {
            Some(b) => b,
            None => false,
        }
    }
}
