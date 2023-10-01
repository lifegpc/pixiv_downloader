use super::context::ServerContext;
use super::params::RequestParams;
use super::preclude::HttpBodyType;
use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use json::JsonValue;
use std::pin::Pin;
use std::sync::Arc;

pub trait MatchRoute<T, R> {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &Request<T>,
    ) -> Option<Box<dyn ResponseFor<T, R> + Send + Sync>>;
}

#[async_trait]
pub trait ResponseFor<T, R> {
    async fn response(&self, req: Request<T>) -> Result<Response<R>, PixivDownloaderError>;
}

#[async_trait]
pub trait ResponseJsonFor<T> {
    async fn response_json(
        &self,
        req: Request<T>,
    ) -> Result<Response<JsonValue>, PixivDownloaderError>;
}

#[async_trait]
/// Get params from request
pub trait GetRequestParams {
    /// Get params from request
    async fn get_params(&mut self) -> Result<RequestParams, PixivDownloaderError>;
}

#[async_trait]
impl<T, U> ResponseFor<T, Pin<Box<HttpBodyType>>> for U
where
    U: ResponseJsonFor<T> + Sync + Send,
    T: Sync + Send + 'static,
{
    async fn response(
        &self,
        req: Request<T>,
    ) -> Result<Response<Pin<Box<HttpBodyType>>>, PixivDownloaderError> {
        let re = self.response_json(req).await?;
        let (mut parts, body) = re.into_parts();
        parts.headers.insert(
            hyper::header::CONTENT_TYPE,
            "application/json; charset=utf-8".parse()?,
        );
        Ok(Response::from_parts(
            parts,
            Box::pin(Body::from(body.to_string())),
        ))
    }
}
