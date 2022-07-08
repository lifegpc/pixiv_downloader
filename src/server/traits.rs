use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use json::JsonValue;

pub trait MatchRoute<T, R> {
    fn get_route(&self) -> Box<dyn ResponseFor<T, R> + Send + Sync>;
    fn match_route(&self, req: &Request<T>) -> bool;
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
impl<T, U> ResponseFor<T, Body> for U
where
    U: ResponseJsonFor<T> + Sync + Send,
    T: Sync + Send + 'static,
{
    async fn response(&self, req: Request<T>) -> Result<Response<Body>, PixivDownloaderError> {
        let re = self.response_json(req).await?;
        let (parts, body) = re.into_parts();
        Ok(Response::from_parts(parts, Body::from(body.to_string())))
    }
}
