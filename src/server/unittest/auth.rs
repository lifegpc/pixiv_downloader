use super::UnitTestContext;
use crate::error::PixivDownloaderError;
use crate::ext::json::FromJson;
use crate::server::result::JSONResult;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use bytes::BytesMut;
use hyper::{Body, Request};

/// Test authentification methods
/// Returns token
pub async fn test(ctx: &UnitTestContext) -> Result<[(u64, Vec<u8>); 2], PixivDownloaderError> {
    let re = ctx
        .request_json2(
            "/auth/user/change/name",
            &json::object! {
                "name": "test",
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(&re)?.unwrap_err();
    assert_eq!(result.code, 19);
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
    let b64_password = base64.encode(&encypted);
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
    let token = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token.len(), 64);
    let token_id = result["id"].as_u64().unwrap();
    let mut password2 = BytesMut::with_capacity(64);
    password2.resize(64, 0);
    openssl::rand::rand_bytes(&mut password2)?;
    let mut encypted2 = BytesMut::with_capacity(tosize);
    encypted2.resize(tosize, 0);
    key.public_encrypt(&password2, &mut encypted2, Padding::PKCS1)?;
    let b64_password2 = base64.encode(&encypted2);
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
                "password" => b64_password2.as_str(),
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
                "password" => b64_password2.as_str(),
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
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test1",
                "password": b64_password2.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add token:");
    assert_eq!(Some(1), result["user_id"].as_u64());
    let token2 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token2.len(), 64);
    let token2_id = result["id"].as_u64().unwrap();
    let re = ctx
        .request_json2_sign(
            "/auth/user/add",
            &json::object! {
                "id" => 1,
                "username" => "test1",
                "name" => "test2",
                "password" => b64_password2.as_str(),
                "is_admin" => false,
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 9);
    let re = ctx
        .request_json2_sign(
            "/auth/user/update",
            &json::object! {
                "id": 1,
                "name": "tesss2",
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "tesss2",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/update",
            &json::object! {
                "id": 1,
                "name": "tesss2",
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 9);
    let re = ctx
        .request_json2_sign(
            "/auth/user/update",
            &json::object! {
                "name": "tesss2",
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 14);
    let re = ctx
        .request_json2_sign(
            "/auth/user/update",
            &json::object! {
                "name": "tess3s2",
                "username": "test1",
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "tess3s2",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/update",
            &json::object! {
                "is_admin": false,
                "username": "test",
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 17);
    let re = ctx
        .request_json2_sign(
            "/auth/user/change/name",
            &json::object! {
                "name": "sdlkasdjklasjd"
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sdlkasdjklasjd",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test1",
                "password": b64_password2.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add token:");
    assert_eq!(Some(1), result["user_id"].as_u64());
    let token3 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token2.len(), 64);
    let token3_id = result["id"].as_u64().unwrap();
    openssl::rand::rand_bytes(&mut password2)?;
    key.public_encrypt(&password2, &mut encypted2, Padding::PKCS1)?;
    let b64_password2 = base64.encode(&encypted2);
    let re = ctx
        .request_json2_sign(
            "/auth/user/change/password",
            &json::object! {
                "password": b64_password2.as_str(),
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sdlkasdjklasjd",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/change/name",
            &json::object! {
                "name": "sadiuqwed",
            },
            &token3,
            token3_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, -403);
    let re = ctx
        .request_json2_sign(
            "/auth/user/change/name",
            &json::object! {
                "name": "sadiuqwed",
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sadiuqwed",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign("/auth/user/info", &json::object! {}, &token2, token2_id)
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sadiuqwed",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/info",
            &json::object! { "id": 1 },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sadiuqwed",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/info",
            &json::object! {"username": "test"},
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 9);
    let re = ctx
        .request_json2_sign(
            "/auth/user/info",
            &json::object! {"username": "test1"},
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 1,
            "name": "sadiuqwed",
            "username": "test1",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/list",
            &json::object! { "id_only": true },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "page": 1,
            "page_count": 10,
            "data": [0, 1],
        }
    );
    let mut password3 = BytesMut::with_capacity(64);
    password3.resize(64, 0);
    openssl::rand::rand_bytes(&mut password3)?;
    let mut encypted3 = BytesMut::with_capacity(tosize);
    encypted3.resize(tosize, 0);
    key.public_encrypt(&password3, &mut encypted3, Padding::PKCS1)?;
    let b64_password3 = base64.encode(&encypted3);
    let re = ctx
        .request_json2_sign(
            "/auth/user/add",
            &json::object! {
                "username": "test2",
                "password": b64_password3.as_str(),
                "name": "test2",
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
            "id": 2,
            "name": "test2",
            "username": "test2",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test2",
                "password": b64_password3.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(Some(2), result["user_id"].as_u64());
    let token3_id = result["id"].as_u64().unwrap();
    let token3 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    let re = ctx
        .request_json2_sign(
            "/auth/user/info",
            &json::object! {"username": "test2"},
            &token3,
            token3_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "id": 2,
            "name": "test2",
            "username": "test2",
            "is_admin": false,
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/list",
            &json::object! { "id_only": true, "page_count": 2, "page": 2 },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "page": 2,
            "page_count": 2,
            "data": [2],
        }
    );
    let re = ctx
        .request_json2_sign(
            "/auth/user/delete",
            &json::object! { "id": 2 },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap_err();
    assert_eq!(result.code, 9);
    let re = ctx
        .request_json2_sign(
            "/auth/user/delete",
            &json::object! { "id": 2 },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(result.as_bool(), Some(true));
    let re = ctx
        .request_json2_sign(
            "/auth/user/list",
            &json::object! { "id_only": true, "page_count": 2, "page": 1 },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "page": 1,
            "page_count": 2,
            "data": [0, 1],
        }
    );
    let re = ctx
        .request_json2_sign("/auth/user/list", &json::object! {}, &token, token_id)
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(
        result,
        json::object! {
            "page": 1,
            "page_count": 10,
            "data": [
                {
                    "id": 0,
                    "name": "test",
                    "username": "test",
                    "is_admin": true,
                },
                {
                    "id": 1,
                    "name": "sadiuqwed",
                    "username": "test1",
                    "is_admin": false,
                },
            ],
        }
    );
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test1",
                "password": b64_password2.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add token:");
    assert_eq!(Some(1), result["user_id"].as_u64());
    let token3 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token3.len(), 64);
    let token3_id = result["id"].as_u64().unwrap();
    let re = ctx
        .request_json2_sign(
            "/auth/token/delete",
            &json::object! {
                "id": [token3_id, token_id],
            },
            &token2,
            token2_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    let result2 = JSONResult::from_json(&result[format!("{}", token3_id)])?.unwrap();
    assert_eq!(result2.as_bool(), Some(true));
    let result2 = JSONResult::from_json(&result[format!("{}", token_id)])?.unwrap_err();
    assert_eq!(result2.code, 13);
    let re = ctx
        .request_json2(
            "/auth/token/add",
            &json::object! {
                "username": "test1",
                "password": b64_password2.as_str(),
            },
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.expect("Failed to add token:");
    assert_eq!(Some(1), result["user_id"].as_u64());
    let token3 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token3.len(), 64);
    let token3_id = result["id"].as_u64().unwrap();
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
    let token4 = base64.decode(result["token"].as_str().unwrap()).unwrap();
    assert_eq!(token4.len(), 64);
    let token4_id = result["id"].as_u64().unwrap();
    let re = ctx
        .request_json2_sign(
            "/auth/token/delete",
            &json::object! {
                "id": [token3_id, token4_id],
            },
            &token,
            token_id,
        )
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    let result2 = JSONResult::from_json(&result[format!("{}", token3_id)])?.unwrap();
    assert_eq!(result2.as_bool(), Some(true));
    let result2 = JSONResult::from_json(&result[format!("{}", token4_id)])?.unwrap();
    assert_eq!(result2.as_bool(), Some(true));
    let re = ctx
        .request_json2_sign("/auth/token/extend", &json::object! {}, &token2, token2_id)
        .await?
        .unwrap();
    let result = JSONResult::from_json(re)?.unwrap();
    assert_eq!(result["user_id"].as_u64(), Some(1));
    assert_eq!(result["id"].as_u64(), Some(token2_id));
    Ok([(token_id, token), (token2_id, token2)])
}
