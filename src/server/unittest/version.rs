use super::super::version::VERSION;
use super::UnitTestContext;
use crate::error::PixivDownloaderError;
use hyper::{Body, Request};

pub async fn test(ctx: &UnitTestContext) -> Result<(), PixivDownloaderError> {
    let re = Request::builder().uri("/version").body(Body::empty())?;
    let res = ctx.request_json(re).await?.unwrap();
    assert_eq!(res, json::object! {"version": VERSION.to_vec()});
    Ok(())
}
