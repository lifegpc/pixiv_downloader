/// Routes about authentication
pub mod auth;
/// Body types which implements [hyper::body::HttpBody]
pub mod body;
pub mod context;
/// CORS Handle
pub mod cors;
/// Get params from request
pub mod params;
/// Predefined includes
pub mod preclude;
/// Routes about proxy
pub mod proxy;
/// Base result type for JSON response
pub mod result;
/// Routes
pub mod route;
/// Services
pub mod service;
/// Tasks invoked by timer
pub mod timer;
/// Traits
pub mod traits;
#[cfg(all(test, feature = "db_sqlite"))]
mod unittest;
/// Version
pub mod version;
