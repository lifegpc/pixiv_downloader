use super::context::ServerContext;
use super::route::ResponseForType;
use super::traits::MatchRoute;
use super::traits::ResponseJsonFor;
use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use json::JsonValue;
use proc_macros::filter_http_methods;
use regex::Regex;
use std::sync::Arc;

pub struct VersionContext {
    ctx: Arc<ServerContext>,
}

impl VersionContext {
    pub fn new(ctx: Arc<ServerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for VersionContext {
    async fn response_json(
        &self,
        req: Request<Body>,
    ) -> Result<Response<JsonValue>, PixivDownloaderError> {
        filter_http_methods!(
            req,
            json::object! {},
            true,
            self.ctx,
            allow_headers = [X_TOKEN],
            GET,
            OPTIONS,
            POST
        );
        Ok(builder.body(json::object! {"version": [0, 0, 1, 0]})?)
    }
}

pub struct VersionRoute {
    regex: Regex,
}

impl VersionRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+version(/.*)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for VersionRoute {
    fn get_route(&self, ctx: Arc<ServerContext>) -> Box<ResponseForType> {
        Box::new(VersionContext::new(ctx))
    }

    fn match_route(&self, req: &http::Request<Body>) -> bool {
        self.regex.is_match(req.uri().path())
    }
}
