use super::auth::*;
use super::context::ServerContext;
use super::traits::MatchRoute;
use super::traits::ResponseFor;
use super::version::VersionRoute;
use hyper::Body;
use hyper::Request;
use std::sync::Arc;

pub type RouteType = dyn MatchRoute<Body, Body> + Send + Sync;
pub type ResponseForType = dyn ResponseFor<Body, Body> + Send + Sync;

pub struct ServerRoutes {
    routes: Vec<Box<RouteType>>,
}

impl ServerRoutes {
    pub fn new() -> Self {
        let mut routes: Vec<Box<RouteType>> = Vec::new();
        routes.push(Box::new(VersionRoute::new()));
        routes.push(Box::new(AuthStatusRoute::new()));
        Self { routes }
    }

    pub fn match_route(
        &self,
        req: &Request<Body>,
        ctx: &Arc<ServerContext>,
    ) -> Option<Box<ResponseForType>> {
        for i in self.routes.iter() {
            if i.match_route(req) {
                return Some(i.get_route(Arc::clone(ctx)));
            }
        }
        None
    }
}
