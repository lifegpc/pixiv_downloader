use super::cors::CorsContext;
use crate::db::{open_database, PixivDownloaderDb};
use crate::gettext;
use std::default::Default;

pub struct ServerContext {
    pub cors: CorsContext,
    pub db: Box<dyn PixivDownloaderDb + Send + Sync>,
}

impl Default for ServerContext {
    fn default() -> Self {
        Self {
            cors: CorsContext::default(),
            db: match open_database() {
                Ok(db) => db,
                Err(e) => panic!("{} {}", gettext("Failed to open database:"), e),
            },
        }
    }
}
