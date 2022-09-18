use super::cors::CorsContext;
use crate::db::{open_and_init_database, PixivDownloaderDb};
use crate::gettext;

pub struct ServerContext {
    pub cors: CorsContext,
    pub db: Box<dyn PixivDownloaderDb + Send + Sync>,
}

impl ServerContext {
    pub async fn default() -> Self {
        Self {
            cors: CorsContext::default(),
            db: match open_and_init_database().await {
                Ok(db) => db,
                Err(e) => panic!("{} {}", gettext("Failed to open database:"), e),
            },
        }
    }
}
