/// Routes about authentication
pub mod auth;
pub mod context;
/// CORS Handle
pub mod cors;
/// Get params from request
pub mod params;
/// Predefined includes
pub mod preclude;
/// Base result type for JSON response
pub mod result;
/// Routes
pub mod route;
/// Services
pub mod service;
/// Traits
pub mod traits;
#[cfg(all(test, feature = "db_sqlite"))]
mod unittest;
/// Version
pub mod version;
