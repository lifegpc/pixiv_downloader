mod auth;
mod version;

use super::context::ServerContext;
use super::cors::CorsContext;
use super::preclude::HttpBodyType;
use super::route::ServerRoutes;
use crate::db::{open_and_init_database, PixivDownloaderDbConfig};
use crate::error::PixivDownloaderError;
use crate::tmp_cache::TmpCache;
use futures_util::lock::Mutex;
use hyper::{Body, Request, Response};
use json::JsonValue;
use std::collections::BTreeMap;
#[cfg(test)]
use std::fs::{create_dir, remove_file};
#[cfg(test)]
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

pub struct UnitTestContext {
    ctx: Arc<ServerContext>,
    routes: ServerRoutes,
}

impl UnitTestContext {
    pub async fn new() -> Self {
        let db = Arc::new(
            open_and_init_database(
                PixivDownloaderDbConfig::new(&json::object! {
                    "type": "sqlite",
                    "path": "test/server.db",
                })
                .unwrap(),
            )
            .await
            .unwrap(),
        );
        Self {
            ctx: Arc::new(ServerContext {
                cors: CorsContext::new(true, vec![], vec![]),
                db: db.clone(),
                rsa_key: Mutex::new(None),
                tmp_cache: Arc::new(TmpCache::new(db.clone())),
                _pixiv_app_client: Mutex::new(None),
                _pixiv_web_client: Mutex::new(None),
            }),
            routes: ServerRoutes::new(),
        }
    }

    pub async fn request(
        &self,
        req: Request<Body>,
    ) -> Result<Option<Response<Pin<Box<HttpBodyType>>>>, PixivDownloaderError> {
        Ok(match self.routes.match_route(&req, &self.ctx) {
            Some(r) => Some(r.response(req).await?),
            None => None,
        })
    }

    pub async fn request_json(
        &self,
        req: Request<Body>,
    ) -> Result<Option<JsonValue>, PixivDownloaderError> {
        Ok(match self.request(req).await? {
            Some(r) => Some(json::parse(&String::from_utf8_lossy(
                &hyper::body::to_bytes(r.into_body()).await?,
            ))?),
            None => None,
        })
    }

    pub async fn request_json2(
        &self,
        uri: &str,
        params: &JsonValue,
    ) -> Result<Option<JsonValue>, PixivDownloaderError> {
        let mut par = Vec::new();
        for (key, obj) in params.entries() {
            if let Some(s) = obj.as_str() {
                par.push(format!(
                    "{}={}",
                    urlparse::quote_plus(key, b"")?,
                    urlparse::quote_plus(s, b"")?
                ));
            } else if obj.is_array() {
                for s in obj.members() {
                    if let Some(s) = s.as_str() {
                        par.push(format!(
                            "{}={}",
                            urlparse::quote_plus(key, b"")?,
                            urlparse::quote_plus(s, b"")?
                        ));
                    } else {
                        par.push(format!(
                            "{}={}",
                            urlparse::quote_plus(key, b"")?,
                            urlparse::quote_plus(&(s.dump()), b"")?
                        ));
                    }
                }
            } else {
                par.push(format!(
                    "{}={}",
                    urlparse::quote_plus(key, b"")?,
                    urlparse::quote_plus(&(obj.dump()), b"")?
                ));
            }
        }
        let par = par.join("&");
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(par))?;
        self.request_json(req).await
    }

    pub async fn request_json2_sign(
        &self,
        uri: &str,
        params: &JsonValue,
        token: &[u8],
        token_id: u64,
    ) -> Result<Option<JsonValue>, PixivDownloaderError> {
        let mut par = BTreeMap::new();
        par.insert(
            "t".to_string(),
            vec![chrono::Utc::now().timestamp().to_string()],
        );
        for (key, obj) in params.entries() {
            if !par.contains_key(key) {
                par.insert(key.to_string(), Vec::new());
            }
            let pp = par.get_mut(key).unwrap();
            if let Some(s) = obj.as_str() {
                pp.push(s.to_owned());
            } else if obj.is_array() {
                for s in obj.members() {
                    if let Some(s) = s.as_str() {
                        pp.push(s.to_owned());
                    } else {
                        pp.push(s.dump());
                    }
                }
            } else {
                pp.push(obj.dump());
            }
        }
        let mut sha = openssl::sha::Sha512::new();
        sha.update(token);
        let mut par2 = Vec::new();
        for (key, value) in par.iter() {
            for v in value {
                sha.update(key.as_bytes());
                sha.update(v.as_bytes());
                par2.push(format!(
                    "{}={}",
                    urlparse::quote_plus(key, b"")?,
                    urlparse::quote_plus(v, b"")?
                ));
            }
        }
        let par2 = par2.join("&");
        let sign = hex::encode(sha.finish());
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("X-SIGN", sign)
            .header("X-TOKEN-ID", token_id.to_string())
            .body(Body::from(par2))?;
        self.request_json(req).await
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test() -> Result<(), PixivDownloaderError> {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let t = Path::new("./test/server.db");
    if t.exists() {
        remove_file(t)?;
    }
    let ctx = UnitTestContext::new().await;
    version::test(&ctx).await?;
    auth::test(&ctx).await?;
    Ok(())
}
