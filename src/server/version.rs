use super::preclude::*;

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
            allow_headers = [CONTENT_TYPE],
            GET,
            OPTIONS,
            POST,
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
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &http::Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        if self.regex.is_match(req.uri().path()) {
            Some(Box::new(VersionContext::new(Arc::clone(ctx))))
        } else {
            None
        }
    }
}
