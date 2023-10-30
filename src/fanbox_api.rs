use crate::ext::atomic::AtomicQuick;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::fanbox::creator::FanboxCreator;
use crate::fanbox::item_list::FanboxItemList;
use crate::fanbox::paginated_creator_posts::PaginatedCreatorPosts;
use crate::fanbox::plan::FanboxPlanList;
use crate::fanbox::post::FanboxPost;
use crate::gettext;
use crate::opthelper::get_helper;
use crate::parser::metadata::MetaDataParser;
use crate::webclient::WebClient;
use json::JsonValue;
use proc_macros::fanbox_api_quick_test;
use reqwest::IntoUrl;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock;

/// Fanbox API client
pub struct FanboxClientInternal {
    /// Web client
    client: Arc<WebClient>,
    /// true if in is initialized
    inited: AtomicBool,
    /// Fanbox global data
    data: RwLock<Option<JsonValue>>,
}

macro_rules! handle_data {
    ($get_data:expr, $err:expr, $info:expr) => {
        match $get_data.await {
            Some(r) => {
                let status = r.status();
                if status.as_u16() >= 400 {
                    log::error!(target: "fanbox_api", "{} {}", $err, status);
                    return None;
                }
                match r.text_with_charset("UTF-8").await {
                    Ok(data) => match json::parse(data.as_str()) {
                        Ok(obj) => {
                            log::debug!(target: "fanbox_api", "{}\n{}", $info, obj.pretty(2),);
                            Some(obj)
                        }
                        Err(e) => {
                            log::error!(target: "fanbox_api", "{} {}", $err, e);
                            None
                        }
                    },
                    Err(e) => {
                        log::error!(target: "fanbox_api", "{} {}", $err, e);
                        None
                    }
                }
            }
            None => None,
        }
    };
}

impl FanboxClientInternal {
    /// Create an new instance
    pub fn new() -> Self {
        Self {
            client: Arc::new(WebClient::default()),
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
        let helper = get_helper();
        self.client.set_header("User-Agent", &helper.user_agent());
        self.client.set_header("referer", "https://www.fanbox.cc/");
        self.client.set_header("origin", "https://www.fanbox.cc");
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
                    log::error!(target: "fanbox_api", "{} {}", gettext("Failed to get fanbox main page:"), status);
                    return false;
                }
                match r.text_with_charset("UTF-8").await {
                    Ok(data) => {
                        let mut parser = MetaDataParser::new("metadata");
                        if !parser.parse(data.as_str()) {
                            log::error!(target: "fanbox_api", "{}", gettext("Failed to parse fanbox main page."));
                            return false;
                        }
                        log::debug!(
                            target: "fanbox_api",
                            "{}\n{}",
                            gettext("Fanbox main page's data:"),
                            parser.value.as_ref().unwrap().pretty(2).as_str()
                        );
                        self.data.replace_with2(parser.value);
                        true
                    }
                    Err(e) => {
                        log::error!(target: "fanbox_api", "{} {}", gettext("Failed to get fanbox main page:"), e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    #[allow(dead_code)]
    /// Get creator's info
    /// * `creator_id` - The id of the creator
    pub async fn get_creator<S: AsRef<str> + ?Sized>(&self, creator_id: &S) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get_with_param(
                "https://api.fanbox.cc/creator.get",
                json::object! {"creatorId": creator_id.as_ref()},
                None,
            ),
            gettext("Failed to get creator's info:"),
            gettext("Creator's info:")
        )
    }

    #[allow(dead_code)]
    /// Get post info
    /// * `post_id` - The id of the post
    pub async fn get_post_info(&self, post_id: u64) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get_with_param(
                "https://api.fanbox.cc/post.info",
                json::object! { "postId": post_id },
                None,
            ),
            gettext("Failed to get post info:"),
            gettext("Post info:")
        )
    }

    /// Send requests to speicfied url.
    /// * `url` - The url
    /// * `errmsg` - The error message when error occured.
    /// * `infomsg` - Verbose output info message.
    pub async fn get_url<U: IntoUrl + Clone, S: AsRef<str> + ?Sized, I: AsRef<str> + ?Sized>(
        &self,
        url: U,
        errmsg: &S,
        infomsg: &I,
    ) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get(url, None),
            errmsg.as_ref(),
            infomsg.as_ref()
        )
    }

    #[allow(dead_code)]
    /// List home page's post list. All supported and followed creators' posts are included.
    /// * `limit` - The max count. 10 is used on Fanbox website.
    pub async fn list_home_post(&self, limit: u64) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get_with_param(
                "https://api.fanbox.cc/post.listHome",
                json::object! {"limit": limit},
                None,
            ),
            gettext("Failed to list home page's posts:"),
            gettext("Home page's posts:")
        )
    }

    #[allow(dead_code)]
    /// List all supporting plans.
    pub async fn list_supporting_plan(&self) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client
                .get("https://api.fanbox.cc/plan.listSupporting", None),
            gettext("Failed to list all supporting plans."),
            gettext("All supporting plans:")
        )
    }

    #[allow(dead_code)]
    /// List supported creators' posts.
    /// * `limit` - The max count. 10 is used on Fanbox website.
    pub async fn list_supporting_post(&self, limit: u64) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get_with_param(
                "https://api.fanbox.cc/post.listSupporting",
                json::object! {"limit": limit},
                None,
            ),
            gettext("Failed to list supported creators' posts:"),
            gettext("Supported creators' posts:")
        )
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

    #[allow(dead_code)]
    /// Paginate creator posts
    /// * `creator_id` - The id of the creator
    pub async fn paginate_creator_post<S: AsRef<str> + ?Sized>(
        &self,
        creator_id: &S,
    ) -> Option<JsonValue> {
        self.auto_init();
        handle_data!(
            self.client.get_with_param(
                "https://api.fanbox.cc/post.paginateCreator",
                json::object! {"creatorId": creator_id.as_ref()},
                None,
            ),
            gettext("Failed to paginate creator post:"),
            gettext("Paginated data:")
        )
    }
}

impl AsRef<Arc<WebClient>> for FanboxClientInternal {
    fn as_ref(&self) -> &Arc<WebClient> {
        &self.client
    }
}

/// Fanbox API Client
pub struct FanboxClient {
    /// Internal client
    client: Arc<FanboxClientInternal>,
}

impl FanboxClient {
    #[allow(dead_code)]
    /// Get creator's info
    /// * `creator_id` - The id of the creator
    pub async fn get_creator<S: AsRef<str> + ?Sized>(
        &self,
        creator_id: &S,
    ) -> Option<FanboxCreator> {
        match self.client.get_creator(creator_id).await {
            Some(s) => Some(FanboxCreator::new(&s["body"], Arc::clone(&self.client))),
            None => None,
        }
    }

    #[allow(dead_code)]
    /// Get post info
    /// * `post_id` - The id of the post
    pub async fn get_post_info(&self, post_id: u64) -> Option<FanboxPost> {
        match self.client.get_post_info(post_id).await {
            Some(s) => Some(FanboxPost::new(&s["body"], Arc::clone(&self.client))),
            None => None,
        }
    }

    /// Create an new instance
    pub fn new() -> Self {
        Self {
            client: Arc::new(FanboxClientInternal::new()),
        }
    }

    #[allow(dead_code)]
    /// List home page's post list. All supported and followed creators' posts are included.
    /// * `limit` - The max count. 10 is used on Fanbox website.
    pub async fn list_home_post(&self, limit: u64) -> Option<FanboxItemList> {
        match self.client.list_home_post(limit).await {
            Some(s) => match FanboxItemList::new(&s["body"], Arc::clone(&self.client)) {
                Ok(item) => Some(item),
                Err(e) => {
                    log::error!(target: "fanbox_api", "{}", e);
                    None
                }
            },
            None => None,
        }
    }

    #[allow(dead_code)]
    /// List all supporting plans.
    pub async fn list_supporting_plan(&self) -> Option<FanboxPlanList> {
        match self.client.list_supporting_plan().await {
            Some(s) => match FanboxPlanList::new(&s["body"]) {
                Ok(item) => Some(item),
                Err(e) => {
                    log::error!(target: "fanbox_api", "{}", e);
                    None
                }
            },
            None => None,
        }
    }

    #[allow(dead_code)]
    /// List supported creators' posts.
    /// * `limit` - The max count. 10 is used on Fanbox website.
    pub async fn list_supporting_post(&self, limit: u64) -> Option<FanboxItemList> {
        match self.client.list_supporting_post(limit).await {
            Some(s) => match FanboxItemList::new(&s["body"], Arc::clone(&self.client)) {
                Ok(item) => Some(item),
                Err(e) => {
                    log::error!(target: "fanbox_api", "{}", e);
                    None
                }
            },
            None => None,
        }
    }

    #[allow(dead_code)]
    /// Paginate creator posts
    /// * `creator_id` - The id of the creator
    pub async fn paginate_creator_post<S: AsRef<str> + ?Sized>(
        &self,
        creator_id: &S,
    ) -> Option<PaginatedCreatorPosts> {
        match self.client.paginate_creator_post(creator_id).await {
            Some(s) => match PaginatedCreatorPosts::new(&s, Arc::clone(&self.client)) {
                Ok(re) => Some(re),
                Err(e) => {
                    log::error!(target: "fanbox_api", "{}", e);
                    None
                }
            },
            None => None,
        }
    }
}

impl Deref for FanboxClient {
    type Target = FanboxClientInternal;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

fanbox_api_quick_test!(
    test_list_home_post,
    client.list_home_post(10),
    "Failed to list home page's posts."
);
fanbox_api_quick_test!(
    test_list_supporting_post,
    client.list_supporting_post(10),
    "Failed to list supported creators' posts."
);
fanbox_api_quick_test!(
    test_list_supporting_plan,
    client.list_supporting_plan(),
    "Failed to list all supporting plans."
);
fanbox_api_quick_test!(
    test_get_creator,
    client.get_creator("mozukun43"),
    "Failed to list all supporting plans."
);
