use super::auth::RSAKey;
use super::cors::CorsContext;
use super::params::RequestParams;
use crate::db::{open_and_init_database, PixivDownloaderDb, User};
use crate::error::PixivDownloaderError;
use crate::get_helper;
use crate::gettext;
use futures_util::lock::Mutex;
use hyper::{Body, Request};
use std::collections::BTreeMap;

pub struct ServerContext {
    pub cors: CorsContext,
    pub db: Box<dyn PixivDownloaderDb + Send + Sync>,
    pub rsa_key: Mutex<Option<RSAKey>>,
}

impl ServerContext {
    pub async fn default() -> Self {
        Self {
            cors: CorsContext::default(),
            db: match open_and_init_database(get_helper().db()).await {
                Ok(db) => db,
                Err(e) => panic!("{} {}", gettext("Failed to open database:"), e),
            },
            rsa_key: Mutex::new(None),
        }
    }

    pub async fn verify_token(
        &self,
        req: &Request<Body>,
        params: &RequestParams,
    ) -> Result<User, PixivDownloaderError> {
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
        sha.update(token.token.as_bytes());
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
        Ok(self
            .db
            .get_user(token.user_id)
            .await?
            .ok_or(gettext("No corresponding user was found."))?)
    }
}
