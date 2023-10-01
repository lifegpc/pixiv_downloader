use crate::error::PixivDownloaderError;
use hyper::body::HttpBody;
use reqwest::Response;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct ResponseBody {
    res: Response,
}

impl ResponseBody {
    pub fn new(res: Response) -> Self {
        Self { res }
    }
}

impl HttpBody for ResponseBody {
    type Data = hyper::body::Bytes;
    type Error = PixivDownloaderError;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match Pin::new(&mut Box::pin(self.res.chunk())).poll(cx) {
            Poll::Ready(f) => match f {
                Ok(Some(data)) => Poll::Ready(Some(Ok(data))),
                Ok(None) => Poll::Ready(None),
                Err(e) => Poll::Ready(Some(Err(PixivDownloaderError::from(e)))),
            },
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
