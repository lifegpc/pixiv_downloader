use super::super::preclude::*;
use crate::ext::json::ToJson2;

#[derive(Clone, Debug)]
/// Action to perform on a user.
pub enum AuthUserAction {
    /// Add a new user.
    Add,
}

pub struct AuthUserContext {
    ctx: Arc<ServerContext>,
    action: Option<AuthUserAction>,
    is_restful: bool,
}

impl AuthUserContext {
    pub fn new(ctx: Arc<ServerContext>, action: Option<AuthUserAction>, is_restful: bool) -> Self {
        Self {
            ctx,
            action,
            is_restful,
        }
    }

    async fn handle(&self, mut req: Request<Body>) -> JSONResult {
        let root_user = self.ctx.db.get_user(0).await?;
        let params = req.get_params().await?;
        if root_user.is_some() {
            // # TODO auth
        }
        match &self.action {
            Some(act) => {}
            None => {
                panic!("No action specified for AuthUserContext.");
            }
        }
        Ok(json::object! {})
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthUserContext {
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
        Ok(builder.body(self.handle(req).await.to_json2())?)
    }
}

pub struct AuthUserRoute {
    regex: Regex,
}

impl AuthUserRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+user(/+add)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for AuthUserRoute {
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
                    return Some(Box::new(AuthUserContext::new(
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
                            "add" => Some(AuthUserAction::Add),
                            _ => return None,
                        }
                    }
                    None => {
                        if req.method() == Method::PUT {
                            Some(AuthUserAction::Add)
                        } else {
                            None
                        }
                    }
                };
                Some(Box::new(AuthUserContext::new(
                    Arc::clone(ctx),
                    action,
                    is_restful,
                )))
            }
            None => None,
        }
    }
}
