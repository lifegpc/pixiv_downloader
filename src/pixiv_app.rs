#[cfg(feature = "db")]
use crate::db::PixivDownloaderDb;
use crate::error::PixivDownloaderError;
use crate::ext::atomic::AtomicQuick;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::pixivapp::error::handle_error;
use crate::pixivapp::illust::PixivAppIllust;
use crate::pixivapp::illusts::PixivAppIllusts;
use crate::webclient::{ReqMiddleware, WebClient};
use crate::{get_helper, gettext};
use chrono::{DateTime, Local, SecondsFormat, Utc};
use json::JsonValue;
use reqwest::{Client, IntoUrl, Request, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PixivRestrictType {
    Public,
    Private,
    All,
}

impl ToString for PixivRestrictType {
    fn to_string(&self) -> String {
        match self {
            PixivRestrictType::Public => String::from("public"),
            PixivRestrictType::Private => String::from("private"),
            PixivRestrictType::All => String::from("all"),
        }
    }
}

pub struct PixivAppMiddleware {
    internal: Arc<PixivAppClientInternal>,
}

impl PixivAppMiddleware {
    pub fn new(internal: Arc<PixivAppClientInternal>) -> Self {
        Self { internal }
    }
}

impl ReqMiddleware for PixivAppMiddleware {
    fn handle(&self, r: Request, c: Client) -> Result<Request, PixivDownloaderError> {
        let now = Local::now().to_rfc3339_opts(SecondsFormat::Secs, false);
        let hash = format!(
            "{:x}",
            md5::compute(format!(
                "{}{}",
                now, "28c1fdd170a5204386cb1313c7077b34f83e4aaf4aa829ce78c231e05b0bae2c"
            ))
        );
        log::debug!(target: "pixiv_app", "X-Client-Hash: {}", hash);
        log::debug!(target: "pixiv_app", "X-Client-Time: {}", now);
        let is_app_api = r.url().host_str().unwrap_or("") == "app-api.pixiv.net";
        let mut r = RequestBuilder::from_parts(c, r)
            .header("X-Client-Hash", hash)
            .header("X-Client-Time", now);
        if is_app_api {
            if let Some(token) = self.internal.access_token.get_ref().as_ref() {
                r = r.header("Authorization", format!("Bearer {}", token));
            }
        }
        Ok(r.build()?)
    }
}

pub struct PixivAppClientInternal {
    client: WebClient,
    #[cfg(feature = "db")]
    db: Option<Arc<Box<dyn PixivDownloaderDb + Send + Sync>>>,
    access_token: RwLock<Option<String>>,
    refresh_token: RwLock<Option<String>>,
    /// true if in is initialized
    inited: AtomicBool,
    access_token_expired: RwLock<Option<DateTime<Utc>>>,
}

impl PixivAppClientInternal {
    #[cfg(not(feature = "db"))]
    pub fn new() -> Self {
        Self {
            client: WebClient::default(),
            #[cfg(feature = "db")]
            db: None,
            access_token: RwLock::new(None),
            refresh_token: RwLock::new(None),
            inited: AtomicBool::new(false),
            access_token_expired: RwLock::new(None),
        }
    }

    #[cfg(feature = "db")]
    pub fn with_db(db: Option<Arc<Box<dyn PixivDownloaderDb + Send + Sync>>>) -> Self {
        Self {
            client: WebClient::default(),
            db,
            access_token: RwLock::new(None),
            refresh_token: RwLock::new(None),
            inited: AtomicBool::new(false),
            access_token_expired: RwLock::new(None),
        }
    }

    pub fn is_inited(&self) -> bool {
        self.inited.qload()
    }

    pub async fn init(&self, refresh_token: String) -> bool {
        self.refresh_token.replace_with2(Some(refresh_token));
        let helper = get_helper();
        self.client
            .set_header("User-Agent", "PixivIOSApp/7.16.16 (iOS 16.6; iPad13,18)");
        self.client.set_header("App-OS", "iOS");
        self.client.set_header("App-OS-Version", "16.6");
        self.client.set_header("App-Version", "7.16.16");
        self.client.set_header("Accept", "*/*");
        self.client.set_header(
            "Accept-Language",
            &helper.language().unwrap_or(String::from("ja")),
        );
        #[cfg(feature = "db")]
        {
            if let Some(db) = &self.db {
                if let Ok(Some(token)) = db.get_config("access_token").await {
                    if let Ok(Some(expired)) = db.get_config("access_token_expired").await {
                        if let Ok(expired) = DateTime::parse_from_rfc3339(&expired) {
                            if Local::now() < expired {
                                self.access_token.replace_with2(Some(token));
                                self.access_token_expired
                                    .replace_with2(Some(expired.with_timezone(&Utc)));
                            }
                        };
                    }
                }
            }
        }
        self.inited.qstore(true);
        true
    }

    async fn auto_init(&self) {
        if !self.is_inited() {
            let helper = get_helper();
            let r = self
                .init(
                    helper
                        .refresh_token()
                        .expect(gettext("refresh_token not setted.")),
                )
                .await;
            if !r {
                panic!("{}", gettext("Failed to initialize pixiv app api client."));
            }
        }
    }

    fn get_access_token_expired(&self) -> Option<DateTime<Utc>> {
        self.access_token_expired.get_ref().as_ref().cloned()
    }

    async fn auto_handle(&self) -> Result<(), PixivDownloaderError> {
        self.auto_init().await;
        if self.access_token.get_ref().is_some() {
            if let Some(expired) = self.get_access_token_expired() {
                if Local::now() >= expired {
                    self.access_token.get_mut().take();
                    self.access_token_expired.get_mut().take();
                    self.auth_token().await?;
                }
            }
        } else {
            self.auth_token().await?;
        }
        Ok(())
    }

    fn get_refresh_token(&self) -> Result<String, PixivDownloaderError> {
        match self.refresh_token.get_ref().as_ref() {
            Some(r) => {
                return Ok(r.clone());
            }
            None => Err(PixivDownloaderError::from(gettext(
                "refresh_token not setted.",
            ))),
        }
    }

    async fn auth_token(&self) -> Result<(), PixivDownloaderError> {
        self.auto_init().await;
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(
            String::from("client_id"),
            String::from("KzEZED7aC0vird8jWyHM38mXjNTY"),
        );
        params.insert(String::from("refresh_token"), self.get_refresh_token()?);
        params.insert(String::from("include_policy"), String::from("true"));
        params.insert(
            String::from("client_secret"),
            String::from("W9JZoJe00qPvJsiyCGT3CCtC6ZUtdpKpzMbNlUGP"),
        );
        params.insert(String::from("grant_type"), String::from("refresh_token"));
        let now = Local::now();
        let re = self
            .client
            .post(
                "https://oauth.secure.pixiv.net/auth/token",
                None,
                Some(params),
            )
            .await
            .ok_or(gettext("Failed to get auth token."))?;
        let status = re.status();
        if status.is_success() {
            let obj = json::parse(&re.text().await?)?;
            let access_token = obj["access_token"]
                .as_str()
                .or_else(|| obj["response"]["access_token"].as_str())
                .ok_or("Access token not found.")?;
            let expires_in = obj["expires_in"]
                .as_i64()
                .or_else(|| obj["response"]["expires_in"].as_i64())
                .ok_or("expires_in not found.")?;
            let expired = now + chrono::Duration::seconds(expires_in);
            self.access_token
                .replace_with2(Some(access_token.to_string()));
            self.access_token_expired
                .replace_with2(Some(expired.with_timezone(&Utc)));
            #[cfg(feature = "db")]
            {
                if let Some(db) = &self.db {
                    db.set_config("access_token", access_token).await?;
                    db.set_config("access_token_expired", &expired.to_rfc3339())
                        .await?;
                }
            }
        } else {
            if let Ok(obj) = json::parse(&re.text().await?) {
                if let Some(err) = obj["error_description"].as_str() {
                    return Err(PixivDownloaderError::from(err));
                }
            }
            return Err(PixivDownloaderError::from(status.as_str()));
        }
        Ok(())
    }

    pub async fn get_follow(
        &self,
        restrict: &PixivRestrictType,
    ) -> Result<JsonValue, PixivDownloaderError> {
        self.auto_handle().await?;
        let re = self
            .client
            .get_with_param(
                "https://app-api.pixiv.net/v2/illust/follow",
                json::object! {"restrict": restrict.to_string()},
                None,
            )
            .await
            .ok_or(gettext("Failed to get follow."))?;
        let obj = handle_error(re).await?;
        log::debug!(
            "{}{}",
            gettext("Follower's new illusts: "),
            obj.pretty(2).as_str()
        );
        Ok(obj)
    }

    pub async fn get_illust_details(&self, id: u64) -> Result<JsonValue, PixivDownloaderError> {
        self.auto_handle().await?;
        let re = self
            .client
            .get_with_param(
                "https://app-api.pixiv.net/v1/illust/detail",
                json::object! {"illust_id": id},
                None,
            )
            .await
            .ok_or(gettext("Failed to get illust details."))?;
        let obj = handle_error(re).await?;
        log::debug!("{}{}", gettext("Illust details:"), obj.pretty(2).as_str());
        Ok(obj)
    }

    pub async fn add_illust_to_browsing_history(
        &self,
        ids: Vec<u64>,
    ) -> Result<(), PixivDownloaderError> {
        self.auto_handle().await?;
        let params: Vec<_> = ids.iter().map(|id| ("illust_ids[]", id)).collect();
        let re = self
            .client
            .post(
                "https://app-api.pixiv.net/v2/user/browsing-history/illust/add",
                None,
                Some(params),
            )
            .await
            .ok_or("Failed to add illust to browsing history.")?;
        handle_error(re).await?;
        Ok(())
    }

    pub async fn get_url<U: IntoUrl + Clone, E, I: std::fmt::Display + ?Sized>(
        &self,
        url: U,
        err: E,
        info: &I,
    ) -> Result<JsonValue, PixivDownloaderError>
    where
        PixivDownloaderError: From<E>,
    {
        self.auto_handle().await?;
        let re = self.client.get(url, None).await.ok_or(err)?;
        let obj = handle_error(re).await?;
        log::debug!("{}{}", info, obj.pretty(2).as_str());
        Ok(obj)
    }

    pub async fn get_user_illusts(&self, uid: u64) -> Result<JsonValue, PixivDownloaderError> {
        self.auto_handle().await?;
        let re = self
            .client
            .get_with_param(
                "https://app-api.pixiv.net/v1/user/illusts",
                json::object! {"type": "illust", "user_id": uid, "filter": "for_ios"},
                None,
            )
            .await
            .ok_or(gettext("Failed to get user's illusts."))?;
        let obj = handle_error(re).await?;
        log::debug!("{}{}", gettext("User's illusts: "), obj.pretty(2).as_str());
        Ok(obj)
    }
}

#[derive(Clone)]
/// Pixiv App Client
pub struct PixivAppClient {
    /// Internal data
    internal: Arc<PixivAppClientInternal>,
}

impl PixivAppClient {
    #[cfg(not(feature = "db"))]
    pub fn new() -> Self {
        let r = Self {
            internal: Arc::new(PixivAppClientInternal::new()),
        };
        r.internal
            .client
            .add_req_middleware(Box::new(PixivAppMiddleware::new(r.internal.clone())));
        r
    }

    #[cfg(feature = "db")]
    pub fn with_db(db: Option<Arc<Box<dyn PixivDownloaderDb + Send + Sync>>>) -> Self {
        let r = Self {
            internal: Arc::new(PixivAppClientInternal::with_db(db)),
        };
        r.internal
            .client
            .add_req_middleware(Box::new(PixivAppMiddleware::new(r.internal.clone())));
        r
    }

    pub async fn get_illust_details(
        &self,
        id: u64,
    ) -> Result<PixivAppIllust, PixivDownloaderError> {
        let obj = self.internal.get_illust_details(id).await?;
        Ok(PixivAppIllust::new(obj["illust"].clone()))
    }

    pub async fn get_follow(
        &self,
        restrict: &PixivRestrictType,
    ) -> Result<PixivAppIllusts, PixivDownloaderError> {
        let obj = self.internal.get_follow(restrict).await?;
        PixivAppIllusts::new(self.internal.clone(), obj)
    }

    pub async fn get_user_illusts(
        &self,
        uid: u64,
    ) -> Result<PixivAppIllusts, PixivDownloaderError> {
        let obj = self.internal.get_user_illusts(uid).await?;
        PixivAppIllusts::new(self.internal.clone(), obj)
    }
}

impl AsRef<PixivAppClientInternal> for PixivAppClient {
    fn as_ref(&self) -> &PixivAppClientInternal {
        &self.internal
    }
}

impl Deref for PixivAppClient {
    type Target = PixivAppClientInternal;
    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}
