use super::auth::RSAKey;
use super::cors::CorsContext;
use super::params::RequestParams;
use crate::db::{open_and_init_database, PixivDownloaderDb};
use crate::error::PixivDownloaderError;
use crate::get_helper;
use crate::gettext;
use futures_util::lock::Mutex;
use hyper::{Body, Request};

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
    ) -> Result<(), PixivDownloaderError> {
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
        let token_id = match token_id {
            Some(token_id) => token_id,
            None => return Err(PixivDownloaderError::from(gettext("Token id not found."))),
        }
        .parse::<u64>()?;
        let token = self
            .db
            .get_token(token_id)
            .await?
            .ok_or(gettext("Token not found."))?;
        Ok(())
    }
}
