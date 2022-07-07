use hyper::server::conn::AddrIncoming;
use hyper::server::Server;
use hyper::service::Service;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

pub struct PixivDownloaderSvc {
    _unused: [u8; 0],
}

impl PixivDownloaderSvc {
    pub fn new() -> Self {
        Self { _unused: [] }
    }
}

impl Service<Request<Body>> for PixivDownloaderSvc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(async { Ok(Response::builder().body(Body::from("hello world")).unwrap()) })
    }
}

pub struct PixivDownloaderMakeSvc {
    _unused: [u8; 0],
}

impl PixivDownloaderMakeSvc {
    pub fn new() -> Self {
        Self { _unused: [] }
    }
}

impl<T> Service<T> for PixivDownloaderMakeSvc {
    type Response = PixivDownloaderSvc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let fut = async move { Ok(PixivDownloaderSvc::new()) };
        Box::pin(fut)
    }
}

/// Start the server
pub fn start_server(
    addr: &SocketAddr,
) -> Result<Server<AddrIncoming, PixivDownloaderMakeSvc>, hyper::Error> {
    Ok(Server::try_bind(addr)?.serve(PixivDownloaderMakeSvc::new()))
}
