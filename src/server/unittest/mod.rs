mod version;

use super::context::ServerContext;
use super::cors::CorsContext;
use super::route::ServerRoutes;
use crate::db::{open_and_init_database, PixivDownloaderDbConfig};
use crate::error::PixivDownloaderError;
use futures_util::lock::Mutex;
use hyper::{Body, Request, Response};
use json::JsonValue;
#[cfg(test)]
use std::fs::{create_dir, remove_file};
#[cfg(test)]
use std::path::Path;
use std::sync::Arc;

pub struct UnitTestContext {
    ctx: Arc<ServerContext>,
    routes: ServerRoutes,
}

impl UnitTestContext {
    pub async fn new() -> Self {
        Self {
            ctx: Arc::new(ServerContext {
                cors: CorsContext::new(true, vec![], vec![]),
                db: open_and_init_database(
                    PixivDownloaderDbConfig::new(&json::object! {
                        "type": "sqlite",
                        "path": "test/server.db",
                    })
                    .unwrap(),
                )
                .await
                .unwrap(),
                rsa_key: Mutex::new(None),
            }),
            routes: ServerRoutes::new(),
        }
    }

    pub async fn request(
        &self,
        req: Request<Body>,
    ) -> Result<Option<Response<Body>>, PixivDownloaderError> {
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
    Ok(())
}
