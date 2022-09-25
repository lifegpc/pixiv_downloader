use super::UnitTestContext;
use crate::error::PixivDownloaderError;
use crate::ext::json::FromJson;
use crate::server::result::JSONResult;
use bytes::BytesMut;
use hyper::{Body, Request};
use openssl::{
    pkey::Public,
    rsa::{Padding, Rsa},
};

/// Test authentification methods
/// Returns token
pub async fn test(ctx: &UnitTestContext) -> Result<BytesMut, PixivDownloaderError> {
    let re = Request::builder().uri("/auth").body(Body::empty())?;
    let res = ctx.request_json(re).await?.unwrap();
    assert_eq!(res["has_root_user"].as_bool(), Some(false));
    let re = Request::builder().uri("/auth/pubkey").body(Body::empty())?;
    let res = ctx.request_json(re).await?.unwrap();
    let result = JSONResult::from_json(res)?.expect("Failed to get public key:");
    let pubkey = result["key"].as_str().expect("No pubkey found.");
    let key = Rsa::public_key_from_pem(pubkey.as_bytes())?;
    let mut password = BytesMut::with_capacity(64);
    password.resize(64, 0);
    openssl::rand::rand_bytes(&mut password)?;
    let tosize = key.size() as usize;
    let mut encypted = BytesMut::with_capacity(tosize);
    encypted.resize(tosize, 0);
    key.public_encrypt(&password, &mut encypted, Padding::PKCS1)?;
    let b64_password = base64::encode(&encypted);
    let re = ctx
        .request_json2(
            "/auth/user/add",
            &json::object! {
                "username" => "test",
                "name" => "test",
                "password" => b64_password,
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add user:");
    Ok(BytesMut::new())
}
