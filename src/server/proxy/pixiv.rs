use super::super::preclude::*;
use http::Uri;

pub struct ProxyPixivContext {
    ctx: Arc<ServerContext>,
}

impl ProxyPixivContext {
    pub fn new(ctx: Arc<ServerContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl ResponseFor<Body, Pin<Box<HttpBodyType>>> for ProxyPixivContext {
    async fn response(
        &self,
        mut req: Request<Body>,
    ) -> Result<Response<Pin<Box<HttpBodyType>>>, PixivDownloaderError> {
        filter_http_methods!(
            req,
            Box::pin(Body::empty()),
            true,
            self.ctx,
            allow_headers = [X_SIGN, X_TOKEN_ID],
            typ_def=Pin<Box<HttpBodyType>>,
            GET,
            OPTIONS
        );
        let params = req.get_params().await?;
        let _ = http_error!(401, self.ctx.verify(&req, &params).await);
        let url = http_error!(params.get("url").ok_or("Url is required."));
        let uri = http_error!(Uri::try_from(url));
        let host = uri.host().ok_or("Host is needed.")?;
        if !host.ends_with(".pximg.net") {
            http_error!(403, Err("Host is not allowed."));
        }
        return Ok(builder.body::<Pin<Box<HttpBodyType>>>(Box::pin(Body::empty()))?);
    }
}

pub struct ProxyPixivRoute {
    regex: Regex,
}

impl ProxyPixivRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+proxy/+pixiv(/.*)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Pin<Box<HttpBodyType>>> for ProxyPixivRoute {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        if self.regex.is_match(req.uri().path()) {
            Some(Box::new(ProxyPixivContext::new(Arc::clone(ctx))))
        } else {
            None
        }
    }
}
