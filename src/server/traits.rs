use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use json::JsonValue;

pub trait MatchRoute<T, R> {
    fn get_route(&self) -> Box<dyn ResponseFor<T, R> + Send + Sync>;
    fn match_route(&self, req: &Request<T>) -> bool;
}

pub trait ResponseFor<T, R> {
    fn response(&self, req: Request<T>) -> Result<Response<R>, PixivDownloaderError>;
}

pub trait ResponseJsonFor<T> {
    fn response_json(&self, req: Request<T>) -> Result<JsonValue, PixivDownloaderError>;
}

impl<T, U> ResponseFor<T, Body> for U
where
    U: ResponseJsonFor<T>,
{
    fn response(&self, req: Request<T>) -> Result<Response<Body>, PixivDownloaderError> {
        let re = self.response_json(req)?;
        Ok(Response::new(Body::from(re.to_string())))
    }
}
