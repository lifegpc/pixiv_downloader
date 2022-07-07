use crate::error::PixivDownloaderError;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use json::JsonValue;

pub trait ResponseFor<T, R> {
    fn response(&self, res: Request<T>) -> Result<Response<R>, PixivDownloaderError>;
}

pub trait ResponseJsonFor<T> {
    fn response_json(&self, res: Request<T>) -> Result<JsonValue, PixivDownloaderError>;
}

impl<T> ResponseFor<T, Body> for T
where
    T: ResponseJsonFor<T>,
{
    fn response(&self, res: Request<T>) -> Result<Response<Body>, PixivDownloaderError> {
        let re = self.response_json(res)?;
        Ok(Response::new(Body::from(re.to_string())))
    }
}
