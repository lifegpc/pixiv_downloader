use super::super::preclude::*;
use crate::db::{PushConfig, PushTaskConfig};
use crate::ext::try_err::TryErr3;

/// Push task manage action
pub enum PushAction {
    /// Add a new push task
    Add,
    /// Change a exist push task
    Change,
    /// Get a exist push task
    Get,
}

pub struct PushContext {
    ctx: Arc<ServerContext>,
    action: Option<PushAction>,
    is_restful: bool,
}

impl PushContext {
    pub fn new(ctx: Arc<ServerContext>, action: Option<PushAction>, is_restful: bool) -> Self {
        Self {
            ctx,
            action,
            is_restful,
        }
    }

    async fn handle(&self, mut req: Request<Body>) -> SerdeJSONResult {
        let params = req
            .get_params()
            .await
            .try_err3(400, "Failed to get parameters:")?;
        let t = self
            .ctx
            .verify(&req, &params)
            .await
            .try_err3(401, "Unauthorized")?;
        if t.is_some_and(|t| !t.is_admin) {
            return Err((403, "Permission denied.").into());
        }
        match &self.action {
            Some(a) => match a {
                PushAction::Add => {
                    let config = params.get("config").ok_or((400, "Missing config."))?;
                    let config: PushTaskConfig =
                        serde_json::from_str(config).try_err3(400, "Failed to parse config:")?;
                    let ttl = params
                        .get_u64("ttl")
                        .try_err3(400, "Bad ttl.")?
                        .unwrap_or(300);
                    let push_configs = params
                        .get("push_configs")
                        .ok_or((400, "Missing push_configs."))?;
                    let push_configs: Vec<PushConfig> = serde_json::from_str(push_configs)
                        .try_err3(400, "Failed to parse push_configs:")?;
                    let re = self
                        .ctx
                        .db
                        .add_push_task(&config, &push_configs, ttl)
                        .await
                        .try_err3(500, "Failed to add push task:")?;
                    Ok(serde_json::to_value(re).try_err3(500, "Failed to serialize result:")?)
                }
                PushAction::Change => {
                    let id = params
                        .get_u64("id")
                        .try_err3(400, "Bad id.")?
                        .try_err3(400, "Missing id.")?;
                    let config = match params.get("config") {
                        Some(v) => Some(
                            serde_json::from_str::<PushTaskConfig>(v)
                                .try_err3(400, "Failed to parse config:")?,
                        ),
                        None => None,
                    };
                    let ttl = params.get_u64("ttl").try_err3(400, "Bad ttl")?;
                    let push_configs = match params.get("push_configs") {
                        Some(v) => Some(
                            serde_json::from_str::<Vec<PushConfig>>(v)
                                .try_err3(400, "Failed to parse push_configs:")?,
                        ),
                        None => None,
                    };
                    let push_configs = match &push_configs {
                        Some(v) => Some(v.as_ref()),
                        None => None,
                    };
                    let re = self
                        .ctx
                        .db
                        .update_push_task(id, config.as_ref(), push_configs, ttl)
                        .await
                        .try_err3(500, "Failed to change push task:")?;
                    Ok(serde_json::to_value(re).try_err3(500, "Failed to serialize result:")?)
                }
                PushAction::Get => {
                    let id = params
                        .get_u64("id")
                        .try_err3(400, "Bad id.")?
                        .try_err3(400, "Missing id.")?;
                    let re = self
                        .ctx
                        .db
                        .get_push_task(id)
                        .await
                        .try_err3(500, "Failed to get push task:")?;
                    Ok(serde_json::to_value(re).try_err3(500, "Failed to serialize result:")?)
                }
            },
            None => {
                panic!("PushContext::handle: action is None")
            }
        }
    }
}

#[async_trait]
impl ResponseFor<Body, Pin<Box<HttpBodyType>>> for PushContext {
    async fn response(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Pin<Box<HttpBodyType>>>, PixivDownloaderError> {
        let builder = if self.is_restful {
            filter_http_methods!(
                req,
                Box::pin(HyperBody::empty()),
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                typ_def = Pin<Box<HttpBodyType>>,
                GET,
                OPTIONS,
                PATCH,
                PUT,
            );
            builder
        } else {
            filter_http_methods!(
                req,
                Box::pin(HyperBody::empty()),
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                typ_def = Pin<Box<HttpBodyType>>,
                GET,
                OPTIONS,
                POST,
            );
            builder
        };
        let re = self.handle(req).await;
        self.ctx.response_serde_json_result(builder, re)
    }
}

pub struct PushRoute {
    regex: Regex,
}

impl PushRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+push(/+(add|change|get))?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Pin<Box<HttpBodyType>>> for PushRoute {
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
                    return Some(Box::new(PushContext::new(
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
                            "add" => Some(PushAction::Add),
                            "change" => Some(PushAction::Change),
                            "get" => Some(PushAction::Get),
                            _ => None,
                        }
                    }
                    None => {
                        let m = req.method();
                        if m == Method::PUT {
                            Some(PushAction::Add)
                        } else if m == Method::GET {
                            Some(PushAction::Get)
                        } else if m == Method::PATCH {
                            Some(PushAction::Change)
                        } else {
                            None
                        }
                    }
                };
                Some(Box::new(PushContext::new(
                    Arc::clone(ctx),
                    action,
                    is_restful,
                )))
            }
            None => None,
        }
    }
}
