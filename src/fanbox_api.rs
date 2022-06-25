use crate::ext::atomic::AtomicQuick;
#[cfg(test)]
use crate::ext::replace::ReplaceWith;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
#[cfg(test)]
use futures_util::lock::Mutex;
use json::JsonValue;
#[cfg(test)]
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
#[cfg(test)]
use std::sync::Arc;
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
    /// * `cookies` - The path to cookies file
    ///
    /// Returns true if successed
    pub fn init(&self, cookies: Option<String>) -> bool {
        match cookies {
            Some(c) => {
                if !self.client.read_cookies(c.as_str()) {
                    return false;
                }
            }
            None => {}
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
            let helper = get_helper();
            let r = self.init(helper.cookies());
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

    /// List home page's post list. All supported and followed creators' posts are included.
    /// * `limit` - The max count. 10 is used on Fanbox website.
    pub async fn list_home(&self, limit: u64) -> Option<JsonValue> {
        self.auto_init();
        match self
            .client
            .get_with_param(
                "https://api.fanbox.cc/post.listHome",
                json::object! {"limit": limit},
                json::object! {"referer": "https://www.fanbox.cc/", "origin": "https://www.fanbox.cc"},
            )
            .await
        {
            Some(r) => {
                let status = r.status();
                if status.as_u16() >= 400 {
                    println!("{} {}", gettext("Failed to list home page's posts:"), status);
                    return None;
                }
                match r.text_with_charset("UTF-8").await {
                    Ok(data) => match json::parse(data.as_str()) {
                        Ok(obj) => Some(obj),
                        Err(e) => {
                            println!("{} {}", gettext("Failed to list home page's posts:"), e);
                            None
                        }
                    },
                    Err(e) => {
                        println!("{} {}", gettext("Failed to list home page's posts:"), e);
                        None
                    }
                }
            }
            None => None,
        }
    }

    /// Returns true if is logged in.
    pub fn logined(&self) -> bool {
        let data = self.data.get_ref();
        if data.is_none() {
            return false;
        }
        let value = data.as_ref().unwrap();
        match value["urlContext"]["user"]["isLoggedIn"].as_bool() {
            Some(b) => b,
            None => false,
        }
    }
}

#[cfg(test)]
lazy_static! {
    #[doc(hidden)]
    static ref TEST_CLIENT: Arc<FanboxClient> = Arc::new(FanboxClient::new());
    #[doc(hidden)]
    /// Used to lock initilize process. The data is set to true after trying check login.
    static ref LOCK: Mutex<bool> = Mutex::new(false);
}

#[cfg(test)]
async fn init_test_client(path: String) -> bool {
    let _ = LOCK.lock().await;
    if !TEST_CLIENT.is_inited() {
        if !TEST_CLIENT.init(Some(path)) {
            return false;
        }
    }
    true
}

#[cfg(test)]
async fn check_login() -> bool {
    let mut ctx = LOCK.lock().await;
    if !ctx.deref() {
        ctx.replace_with(true);
        if !TEST_CLIENT.check_login().await {
            return false;
        }
    }
    TEST_CLIENT.logined()
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_list_home() {
    match std::env::var("FANBOX_COOKIES_FILE") {
        Ok(path) => {
            let re = init_test_client(path).await;
            if !re {
                panic!("Failed to initiailze the client.");
            }
            if !check_login().await {
                println!("The client is not logined. Skip test.");
                return;
            }
            match TEST_CLIENT.list_home(10).await {
                Some(data) => {
                    println!("{}", data.pretty(2));
                }
                None => {
                    panic!("Failed to list home.");
                }
            }
        }
        Err(_) => {
            println!("No cookies files specified, skip test.")
        }
    }
}
