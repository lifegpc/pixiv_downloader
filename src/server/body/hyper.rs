use crate::server::preclude::PixivDownloaderError;
use hyper::body::HttpBody;
use hyper::Body as _Body;
use hyper::HeaderMap;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct HyperBody {
    _body: _Body,
}

impl HyperBody {
    pub fn empty() -> Self {
        Self {
            _body: _Body::empty(),
        }
    }
}

impl<T: Into<_Body>> From<T> for HyperBody {
    fn from(body: T) -> Self {
        Self { _body: body.into() }
    }
}

impl HttpBody for HyperBody {
    type Data = bytes::Bytes;
    type Error = PixivDownloaderError;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut self._body)
            .poll_data(cx)
            .map_err(PixivDownloaderError::from)
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::new(&mut self._body)
            .poll_trailers(cx)
            .map_err(PixivDownloaderError::from)
    }

    fn is_end_stream(&self) -> bool {
        self._body.is_end_stream()
    }
}
