use super::UnitTestContext;
use crate::error::PixivDownloaderError;
use crate::ext::json::FromJson;
use crate::server::result::JSONResult;
use bytes::BytesMut;
use hyper::{Body, Request};
use openssl::rsa::{Padding, Rsa};

/// Test authentification methods
/// Returns token
pub async fn test(ctx: &UnitTestContext) -> Result<(u64, Vec<u8>), PixivDownloaderError> {
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
                "password" => b64_password.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add user:");
    assert_eq!(
        result,
        json::object! {
            "id": 0,
            "name": "test",
            "username": "test",
            "is_admin": true,
        }
    );
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test",
                "password": b64_password.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add token:");
    assert_eq!(Some(0), result["user_id"].as_u64());
    let token = base64::decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token.len(), 64);
    let token_id = result["id"].as_u64().unwrap();
    let mut password2 = BytesMut::with_capacity(64);
    password2.resize(64, 0);
    openssl::rand::rand_bytes(&mut password2)?;
    let mut encypted2 = BytesMut::with_capacity(tosize);
    encypted2.resize(tosize, 0);
    key.public_encrypt(&password2, &mut encypted2, Padding::PKCS1)?;
    let b64_password2 = base64::encode(&encypted2);
    let re = ctx
        .request_json2_sign(
            "/auth/user/add",
            &json::object! {
                "username" => "test2",
                "name" => "test2",
                "password" => b64_password2.as_str(),
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add user:");
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "test2",
            "username": "test2",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/add",
            &json::object! {
                "id" => 1,
                "username" => "test1",
                "name" => "test1",
                "password" => b64_password.as_str(),
                "is_admin" => true,
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add user:");
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "test1",
            "username": "test1",
            "is_admin": true,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/add",
            &json::object! {
                "id" => 1,
                "username" => "test1",
                "name" => "test2",
                "password" => b64_password.as_str(),
                "is_admin" => false,
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add user:");
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "test2",
            "username": "test1",
            "is_admin": false,
        }
    );
    Ok((token_id, token))
}
