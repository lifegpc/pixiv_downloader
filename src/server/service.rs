use super::route::ServerRoutes;
use hyper::server::conn::AddrIncoming;
use hyper::server::Server;
use hyper::service::Service;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

pub struct PixivDownloaderSvc {
    routes: Arc<ServerRoutes>,
}

impl PixivDownloaderSvc {
    pub fn new(routes: Arc<ServerRoutes>) -> Self {
        Self { routes }
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
        match self.routes.match_route(&req) {
            Some(route) => Box::pin(async move {
                match route.response(req) {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        println!("{}", e);
                        Ok(Response::builder()
                            .status(500)
                            .body(Body::from("Internal server error"))
                            .unwrap())
                    }
                }
            }),
            None => {
                Box::pin(async { Ok(Response::builder().body(Body::from("hello world")).unwrap()) })
            }
        }
    }
}

pub struct PixivDownloaderMakeSvc {
    routes: Arc<ServerRoutes>,
}

impl PixivDownloaderMakeSvc {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(ServerRoutes::new()),
        }
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
        let routes = Arc::clone(&self.routes);
        let fut = async move { Ok(PixivDownloaderSvc::new(routes)) };
        Box::pin(fut)
    }
}

/// Start the server
pub fn start_server(
    addr: &SocketAddr,
) -> Result<Server<AddrIncoming, PixivDownloaderMakeSvc>, hyper::Error> {
    Ok(Server::try_bind(addr)?.serve(PixivDownloaderMakeSvc::new()))
}
