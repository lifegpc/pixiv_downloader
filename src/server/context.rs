use super::auth::RSAKey;
use super::body::hyper::HyperBody;
use super::cors::CorsContext;
use super::params::RequestParams;
use super::preclude::HttpBodyType;
use super::result::{JSONResult, SerdeJSONResult, SerdeJSONResult2};
use crate::db::{open_and_init_database, PixivDownloaderDb, Token, User};
use crate::error::PixivDownloaderError;
use crate::ext::json::ToJson2;
use crate::get_helper;
use crate::gettext;
use crate::pixiv_app::PixivAppClient;
use crate::pixiv_web::PixivWebClient;
use futures_util::lock::Mutex;
use hyper::{http::response::Builder, Body, Request, Response};
use json::JsonValue;
use reqwest::IntoUrl;
use std::collections::{BTreeMap, HashMap};
use std::pin::Pin;
use std::sync::Arc;

pub struct ServerContext {
    pub cors: CorsContext,
    pub db: Arc<Box<dyn PixivDownloaderDb + Send + Sync>>,
    pub rsa_key: Mutex<Option<RSAKey>>,
    pub _pixiv_app_client: Mutex<Option<PixivAppClient>>,
    pub _pixiv_web_client: Mutex<Option<Arc<PixivWebClient>>>,
}

impl ServerContext {
    pub async fn default() -> Self {
        Self {
            cors: CorsContext::default(),
            db: match open_and_init_database(get_helper().db()).await {
                Ok(db) => Arc::new(db),
                Err(e) => panic!("{} {}", gettext("Failed to open database:"), e),
            },
            rsa_key: Mutex::new(None),
            _pixiv_app_client: Mutex::new(None),
            _pixiv_web_client: Mutex::new(None),
        }
    }

    pub async fn generate_pixiv_proxy_url<U: IntoUrl>(
        &self,
        u: U,
    ) -> Result<String, PixivDownloaderError> {
        let u = u.into_url()?;
        let host = u.host_str().ok_or("Host not found.")?;
        if !host.ends_with(".pximg.net") {
            return Err("Host not match.".into());
        }
        let helper = get_helper();
        let base = helper
            .server_base()
            .unwrap_or(format!("http://{}", helper.server()));
        let mut map = HashMap::new();
        map.insert("url", u.as_str());
        let secret = self.db.get_proxy_pixiv_secrets().await?;
        let mut sha512 = openssl::sha::Sha512::new();
        sha512.update(secret.as_bytes());
        sha512.update("url".as_bytes());
        sha512.update(u.as_str().as_bytes());
        let sign = hex::encode(sha512.finish());
        map.insert("sign", &sign);
        let url = format!("{}/proxy/pixiv?{}", base, serde_urlencoded::to_string(map)?);
        Ok(url)
    }

    pub async fn pixiv_app_client(&self) -> PixivAppClient {
        let mut pixiv_app_client = self._pixiv_app_client.lock().await;
        if pixiv_app_client.is_none() {
            pixiv_app_client.replace(PixivAppClient::with_db(Some(self.db.clone())));
        }
        pixiv_app_client.as_ref().unwrap().clone()
    }

    pub async fn pixiv_web_client(&self) -> Arc<PixivWebClient> {
        let mut pixiv_web_client = self._pixiv_web_client.lock().await;
        if pixiv_web_client.is_none() {
            pixiv_web_client.replace(Arc::new(PixivWebClient::new()));
        }
        pixiv_web_client.as_ref().unwrap().clone()
    }

    pub fn response_json_result(
        &self,
        builder: Builder,
        re: JSONResult,
    ) -> Result<Response<JsonValue>, PixivDownloaderError> {
        let builder = match &re {
            Ok(_) => builder,
            Err(err) => {
                if err.code <= -400 && err.code >= -600 {
                    builder.status((-err.code) as u16)
                } else if err.code < 0 {
                    builder.status(500)
                } else if err.code > 0 {
                    builder.status(400)
                } else {
                    builder
                }
            }
        };
        Ok(builder.body(re.to_json2())?)
    }

    pub fn response_serde_json_result(
        &self,
        builder: Builder,
        re: SerdeJSONResult,
    ) -> Result<Response<Pin<Box<HttpBodyType>>>, PixivDownloaderError> {
        let builder = match &re {
            Ok(_) => builder,
            Err(err) => {
                if err.code <= -400 && err.code >= -600 {
                    builder.status((-err.code) as u16)
                } else if err.code < 0 {
                    builder.status(500)
                } else if err.code > 0 {
                    builder.status(400)
                } else {
                    builder
                }
            }
        };
        let s = SerdeJSONResult2::new(re);
        Ok(
            builder.body::<Pin<Box<HttpBodyType>>>(Box::pin(HyperBody::from(
                serde_json::to_string(&s)?,
            )))?,
        )
    }

    pub async fn verify_token2(
        &self,
        req: &Request<Body>,
        params: &RequestParams,
    ) -> Result<Token, PixivDownloaderError> {
        let mut token_id = None;
        match req.headers().get("X-TOKEN-ID") {
            Some(v) => {
                token_id.replace(v.to_str()?.to_owned());
            }
            None => match params.get("token_id") {
                Some(v) => {
                    token_id.replace(v.to_owned());
                }
                None => {}
            },
        }
        let mut sign = None;
        match req.headers().get("X-SIGN") {
            Some(v) => {
                sign.replace(v.to_str()?.to_owned());
            }
            None => match params.get("sign") {
                Some(v) => {
                    sign.replace(v.to_owned());
                }
                None => {}
            },
        }
        let token_id = match token_id {
            Some(token_id) => token_id,
            None => return Err(PixivDownloaderError::from(gettext("Token id not found."))),
        }
        .parse::<u64>()?;
        let sign = match sign {
            Some(sign) => sign,
            None => return Err(PixivDownloaderError::from(gettext("Sign not found."))),
        };
        let time = params
            .get_u64_mult(&["time", "t"])?
            .ok_or(gettext("Time not found."))?;
        let now = chrono::Utc::now().timestamp() as u64;
        if time < now - 300 || time > now + 300 {
            return Err(PixivDownloaderError::from(gettext("Time out of range.")));
        }
        let token = self
            .db
            .get_token(token_id)
            .await?
            .ok_or(gettext("Token not found."))?;
        let mut par = BTreeMap::new();
        for (k, v) in params.params.iter() {
            if k == "sign" || k == "token_id" {
                continue;
            }
            par.insert(k, v);
        }
        let mut sha = openssl::sha::Sha512::new();
        sha.update(&token.token);
        for (k, v) in par {
            for v in v {
                sha.update(k.as_bytes());
                sha.update(v.as_bytes());
            }
        }
        let sha = hex::encode(sha.finish());
        if sign != sha {
            return Err(PixivDownloaderError::from(gettext("Sign not match.")));
        }
        Ok(token)
    }

    pub async fn verify_token(
        &self,
        req: &Request<Body>,
        params: &RequestParams,
    ) -> Result<User, PixivDownloaderError> {
        let token = self.verify_token2(req, params).await?;
        Ok(self
            .db
            .get_user(token.user_id)
            .await?
            .ok_or(gettext("No corresponding user was found."))?)
    }

    pub async fn verify_secrets(
        &self,
        req: &Request<Body>,
        params: &RequestParams,
        secrets: String,
        time_needed: bool,
    ) -> Result<(), PixivDownloaderError> {
        let mut sign = None;
        match req.headers().get("X-SIGN") {
            Some(v) => {
                sign.replace(v.to_str()?.to_owned());
            }
            None => match params.get("sign") {
                Some(v) => {
                    sign.replace(v.to_owned());
                }
                None => {}
            },
        }
        let sign = match sign {
            Some(sign) => sign,
            None => return Err(PixivDownloaderError::from(gettext("Sign not found."))),
        };
        if time_needed {
            let time = params
                .get_u64_mult(&["time", "t"])?
                .ok_or(gettext("Time not found."))?;
            let now = chrono::Utc::now().timestamp() as u64;
            if time < now - 300 || time > now + 300 {
                return Err(PixivDownloaderError::from(gettext("Time out of range.")));
            }
        }
        let mut par = BTreeMap::new();
        for (k, v) in params.params.iter() {
            if k == "sign" {
                continue;
            }
            par.insert(k, v);
        }
        let mut sha = openssl::sha::Sha512::new();
        sha.update(secrets.as_bytes());
        for (k, v) in par {
            for v in v {
                sha.update(k.as_bytes());
                sha.update(v.as_bytes());
            }
        }
        let sha = hex::encode(sha.finish());
        if sign != sha {
            return Err(PixivDownloaderError::from(gettext("Sign not match.")));
        }
        Ok(())
    }

    pub async fn verify(
        &self,
        req: &Request<Body>,
        params: &RequestParams,
    ) -> Result<Option<User>, PixivDownloaderError> {
        let root_user = self.db.get_user(0).await?;
        if root_user.is_some() {
            Ok(Some(self.verify_token(req, params).await?))
        } else {
            Ok(None)
        }
    }
}
