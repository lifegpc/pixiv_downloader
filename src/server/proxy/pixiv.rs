use super::super::preclude::*;
use crate::webclient::WebClient;
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
            Box::pin(HyperBody::empty()),
            true,
            self.ctx,
            allow_headers = [X_SIGN, X_TOKEN_ID],
            typ_def=Pin<Box<HttpBodyType>>,
            GET,
            OPTIONS
        );
        let params = req.get_params().await?;
        let secrets = self.ctx.db.get_proxy_pixiv_secrets().await?;
        http_error!(
            401,
            self.ctx.verify_secrets(&req, &params, secrets, false).await
        );
        let url = http_error!(params.get("url").ok_or("Url is required."));
        let uri = http_error!(Uri::try_from(url));
        let host = http_error!(uri.host().ok_or("Host is needed."));
        if !host.ends_with(".pximg.net") {
            http_error!(403, Err("Host is not allowed."));
        }
        let client = WebClient::default();
        client.set_header("referer", "https://www.pixiv.net/");
        let o = req.headers();
        match o.get("user-agent") {
            Some(v) => {
                client.set_header(
                    "user-agent",
                    v.to_str()
                        .unwrap_or("PixivAndroidApp/5.0.234 (Android 11; Pixel 5)"),
                );
            }
            None => {}
        }
        let keys = ["Range", "Accept", "If-Modified-Since"];
        for k in keys {
            match o.get(k) {
                Some(v) => {
                    client.set_header(k, v.to_str().unwrap_or(""));
                }
                None => {}
            }
        }
        let re = http_error!(
            502,
            client.get(url, None).await.ok_or("Failed to get image.")
        );
        builder = builder.status(re.status());
        let keys = [
            "cache-control",
            "content-length",
            "content-type",
            "date",
            "last-modified",
            "content-range",
            "age",
            "expires",
            "keep-alive",
            "location",
            "server",
        ];
        let o = re.headers();
        for k in keys {
            match o.get(k) {
                Some(v) => {
                    builder = builder.header(k, v.to_str().unwrap_or(""));
                }
                None => {}
            }
        }
        return Ok(builder.body::<Pin<Box<HttpBodyType>>>(Box::pin(ResponseBody::new(re)))?);
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
