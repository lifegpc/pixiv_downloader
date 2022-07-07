use super::route::ResponseForType;
use super::traits::MatchRoute;
use super::traits::ResponseJsonFor;
use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use json::JsonValue;
use regex::Regex;

pub struct VersionContext {
    _unused: [u8; 0],
}

impl VersionContext {
    pub fn new() -> Self {
        Self { _unused: [] }
    }
}

impl ResponseJsonFor<Body> for VersionContext {
    fn response_json(&self, _req: Request<Body>) -> Result<JsonValue, PixivDownloaderError> {
        Ok(json::object! {"version": [0, 0, 1, 0]})
    }
}

pub struct VersionRoute {
    regex: Regex,
}

impl VersionRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/api)?/version(/.*)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for VersionRoute {
    fn get_route(&self) -> Box<ResponseForType> {
        Box::new(VersionContext::new())
    }

    fn match_route(&self, req: &http::Request<Body>) -> bool {
        self.regex.is_match(req.uri().path())
    }
}
