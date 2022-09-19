use super::super::preclude::*;

pub struct AuthStatusContext {
    ctx: Arc<ServerContext>,
}

impl AuthStatusContext {
    pub fn new(ctx: Arc<ServerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthStatusContext {
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
        let has_root_user = self.ctx.db.get_user(0).await?.is_some();
        Ok(builder.body(json::object! {"has_root_user": has_root_user})?)
    }
}

pub struct AuthStatusRoute {
    regex: Regex,
}

impl AuthStatusRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth(/+status(/.*)?)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for AuthStatusRoute {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &http::Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        if self.regex.is_match(req.uri().path()) {
            Some(Box::new(AuthStatusContext::new(Arc::clone(ctx))))
        } else {
            None
        }
    }
}
