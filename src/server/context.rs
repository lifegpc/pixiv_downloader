use super::cors::CorsContext;
use std::default::Default;

pub struct ServerContext {
    pub cors: CorsContext,
}

impl Default for ServerContext {
    fn default() -> Self {
        Self {
            cors: CorsContext::default(),
        }
    }
}
