use super::auth::RSAKey;
use super::cors::CorsContext;
use crate::db::{open_and_init_database, PixivDownloaderDb};
use crate::gettext;
use futures_util::lock::Mutex;

pub struct ServerContext {
    pub cors: CorsContext,
    pub db: Box<dyn PixivDownloaderDb + Send + Sync>,
    pub rsa_key: Mutex<Option<RSAKey>>,
}

impl ServerContext {
    pub async fn default() -> Self {
        Self {
            cors: CorsContext::default(),
            db: match open_and_init_database().await {
                Ok(db) => db,
                Err(e) => panic!("{} {}", gettext("Failed to open database:"), e),
            },
            rsa_key: Mutex::new(None),
        }
    }
}
