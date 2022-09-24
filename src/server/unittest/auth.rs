use super::UnitTestContext;
use crate::error::PixivDownloaderError;
use bytes::BytesMut;
use hyper::{Body, Request};

/// Test authentification methods
/// Returns token
pub async fn test(ctx: &UnitTestContext) -> Result<BytesMut, PixivDownloaderError> {
    let re = Request::builder().uri("/auth").body(Body::empty())?;
    let res = ctx.request_json(re).await?.unwrap();
    assert_eq!(res["has_root_user"].as_bool(), Some(false));
    let re = Request::builder().uri("/auth/pubkey").body(Body::empty())?;
    let res = ctx.request_json(re).await?.unwrap();
    let ok = res["ok"].as_bool().unwrap();
    if !ok {
        panic!("Failed to get public key.");
    }
    Ok(BytesMut::new())
}
