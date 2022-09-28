use super::super::preclude::*;

/// Action to perform about token
pub enum AuthTokenAction {
    /// Add a new token
    Add,
}

pub struct AuthTokenContext {
    ctx: Arc<ServerContext>,
    action: Option<AuthTokenAction>,
    is_restful: bool,
}

impl AuthTokenContext {
    pub fn new(ctx: Arc<ServerContext>, action: Option<AuthTokenAction>, is_restful: bool) -> Self {
        Self {
            ctx,
            action,
            is_restful,
        }
    }

    async fn handle(&self, mut req: Request<Body>) -> JSONResult {
        Ok(json::object! {})
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthTokenContext {
    async fn response_json(
        &self,
        req: Request<Body>,
    ) -> Result<Response<JsonValue>, PixivDownloaderError> {
        let builder = if self.is_restful {
            filter_http_methods!(
                req,
                json::object! {},
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                OPTIONS,
                PUT,
            );
            builder
        } else {
            filter_http_methods!(
                req,
                json::object! {},
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                GET,
                OPTIONS,
                POST,
            );
            builder
        };
        let re = self.handle(req).await;
        self.ctx.response_json_result(builder, re)
    }
}

pub struct AuthTokenRoute {
    regex: Regex,
}

impl AuthTokenRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+token(/+add)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for AuthTokenRoute {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &http::Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        let path = req.uri().path();
        let pat = self.regex.captures(path);
        match pat {
            Some(cap) => {
                if req.method() == Method::OPTIONS {
                    return Some(Box::new(AuthTokenContext::new(
                        Arc::clone(ctx),
                        None,
                        cap.get(2).is_none(),
                    )));
                }
                let cap2 = cap.get(2);
                let is_restful = cap2.is_none();
                let action = match cap2 {
                    Some(m) => {
                        let m = m.as_str().trim_start_matches("/");
                        match m {
                            "add" => Some(AuthTokenAction::Add),
                            _ => return None,
                        }
                    }
                    None => {
                        if req.method() == Method::PUT {
                            Some(AuthTokenAction::Add)
                        } else {
                            None
                        }
                    }
                };
                Some(Box::new(AuthTokenContext::new(
                    Arc::clone(ctx),
                    action,
                    is_restful,
                )))
            }
            None => None,
        }
    }
}
