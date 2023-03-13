use super::super::preclude::*;
use super::{PASSWORD_ITER, PASSWORD_SALT};
use crate::db::User;
use crate::ext::json::ToJson2;
use crate::ext::try_err::TryErr3;
use crate::gettext;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use openssl::{hash::MessageDigest, pkcs5::pbkdf2_hmac};

/// Action to perform about token
pub enum AuthTokenAction {
    /// Add a new token
    Add,
    /// Delete a token
    Delete,
    /// Extend a token's expire time
    Extend,
}

pub struct AuthTokenContext {
    ctx: Arc<ServerContext>,
    action: Option<AuthTokenAction>,
    is_restful: bool,
}

impl AuthTokenContext {
    pub fn new(ctx: Arc<ServerContext>, action: Option<AuthTokenAction>, is_restful: bool) -> Self {
        Self {
            ctx,
            action,
            is_restful,
        }
    }

    async fn handle(&self, mut req: Request<Body>) -> JSONResult {
        let params = req
            .get_params()
            .await
            .try_err3(-1002, gettext("Failed to get parameters:"))?;
        match &self.action {
            Some(s) => match s {
                AuthTokenAction::Add => {
                    let username = params
                        .get("username")
                        .ok_or((1, gettext("No username specified.")))?;
                    let password = params
                        .get("password")
                        .ok_or((2, gettext("No password specified.")))?;
                    let password = base64
                        .decode(password)
                        .try_err3(3, gettext("Failed to decode password with base64:"))?;
                    let rsa_key = self.ctx.rsa_key.lock().await;
                    let key = rsa_key.as_ref().ok_or((4, gettext("No RSA key found.")))?;
                    if key.is_too_old() {
                        return Err((
                            5,
                            gettext("RSA key is too old. A new key should be generated."),
                        )
                            .into());
                    }
                    let password = key
                        .decrypt(&password)
                        .try_err3(6, gettext("Failed to decrypt password with RSA:"))?;
                    let mut hashed_password = [0; 64];
                    pbkdf2_hmac(
                        &password,
                        &PASSWORD_SALT,
                        PASSWORD_ITER,
                        MessageDigest::sha512(),
                        &mut hashed_password,
                    )
                    .try_err3(7, gettext("Failed to hash password:"))?;
                    let user = self
                        .ctx
                        .db
                        .get_user_by_username(username)
                        .await
                        .try_err3(-1001, gettext("Failed to operate the database:"))?
                        .ok_or((8, gettext("User not found.")))?;
                    let pass: &[u8] = &user.password;
                    if pass != &hashed_password {
                        return Err((9, gettext("Wrong password.")).into());
                    }
                    let mut token = [0; 64];
                    openssl::rand::rand_bytes(&mut token)
                        .try_err3(-1003, gettext("Failed to generate token:"))?;
                    let created_at = chrono::Utc::now();
                    let expired_at = created_at + chrono::Duration::days(30);
                    let token = loop {
                        if let Some(token) = self
                            .ctx
                            .db
                            .add_token(user.id, &token, &created_at, &expired_at)
                            .await
                            .try_err3(-1001, gettext("Failed to operate the database:"))?
                        {
                            break token;
                        }
                    };
                    let b64token = base64.encode(&token.token);
                    Ok(
                        json::object! { "id": token.id, "user_id": token.user_id, "token": b64token, "created_at": token.created_at.timestamp(), "expired_at": token.expired_at.timestamp() },
                    )
                }
                AuthTokenAction::Delete => {
                    let user = self
                        .ctx
                        .verify_token(&req, &params)
                        .await
                        .try_err3(-403, gettext("Failed to verify the token:"))?;
                    let ids = params
                        .get_u64_all("id")
                        .try_err3(
                            10,
                            &gettext("Failed to parse <opt>:").replace("<opt>", "id"),
                        )?
                        .try_err3(11, gettext("No id specified."))?;
                    let mut data = json::JsonValue::new_object();
                    for id in ids {
                        data.insert(
                            &format!("{}", id),
                            self.revoke_token(id, &user).await.to_json2(),
                        )
                        .try_err3(-1004, gettext("Failed to insert data to JSON:"))?;
                    }
                    Ok(data)
                }
                AuthTokenAction::Extend => {
                    let token = self
                        .ctx
                        .verify_token2(&req, &params)
                        .await
                        .try_err3(-403, gettext("Failed to verify the token:"))?;
                    let expired_at = chrono::Utc::now() + chrono::Duration::days(30);
                    self.ctx
                        .db
                        .extend_token(token.id, &expired_at)
                        .await
                        .try_err3(-1001, gettext("Failed to operate the database:"))?;
                    Ok(
                        json::object! { "id": token.id, "user_id": token.user_id, "created_at": token.created_at.timestamp(), "expired_at": expired_at.timestamp() },
                    )
                }
            },
            None => {
                panic!("No action specified for AuthTokenContext.");
            }
        }
    }

    async fn revoke_token(&self, id: u64, user: &User) -> JSONResult {
        let token = self
            .ctx
            .db
            .get_token(id)
            .await
            .try_err3(-1001, gettext("Failed to operate the database:"))?
            .try_err3(12, gettext("Token not found."))?;
        if token.user_id != user.id && !user.is_admin {
            return Err((13, gettext("Permission denied.")).into());
        }
        self.ctx
            .db
            .delete_token(id)
            .await
            .try_err3(-1001, gettext("Failed to operate the database:"))?;
        Ok(json::JsonValue::Boolean(true))
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthTokenContext {
    async fn response_json(
        &self,
        req: Request<Body>,
    ) -> Result<Response<JsonValue>, PixivDownloaderError> {
        let builder = if self.is_restful {
            filter_http_methods!(
                req,
                json::object! {},
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                OPTIONS,
                PUT,
                DELETE,
            );
            builder
        } else {
            filter_http_methods!(
                req,
                json::object! {},
                true,
                self.ctx,
                allow_headers = [CONTENT_TYPE, X_SIGN, X_TOKEN_ID],
                GET,
                OPTIONS,
                POST,
            );
            builder
        };
        let re = self.handle(req).await;
        self.ctx.response_json_result(builder, re)
    }
}

pub struct AuthTokenRoute {
    regex: Regex,
}

impl AuthTokenRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+token(/+(add|delete|extend))?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for AuthTokenRoute {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &http::Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        let path = req.uri().path();
        let pat = self.regex.captures(path);
        match pat {
            Some(cap) => {
                if req.method() == Method::OPTIONS {
                    return Some(Box::new(AuthTokenContext::new(
                        Arc::clone(ctx),
                        None,
                        cap.get(2).is_none(),
                    )));
                }
                let cap2 = cap.get(2);
                let is_restful = cap2.is_none();
                let action = match cap2 {
                    Some(m) => {
                        let m = m.as_str().trim_start_matches("/");
                        match m {
                            "add" => Some(AuthTokenAction::Add),
                            "delete" => Some(AuthTokenAction::Delete),
                            "extend" => Some(AuthTokenAction::Extend),
                            _ => return None,
                        }
                    }
                    None => {
                        let m = req.method();
                        if m == Method::PUT {
                            Some(AuthTokenAction::Add)
                        } else if m == Method::DELETE {
                            Some(AuthTokenAction::Delete)
                        } else {
                            None
                        }
                    }
                };
                Some(Box::new(AuthTokenContext::new(
                    Arc::clone(ctx),
                    action,
                    is_restful,
                )))
            }
            None => None,
        }
    }
}

pub async fn revoke_expired_tokens(ctx: Arc<ServerContext>) -> Result<(), PixivDownloaderError> {
    ctx.db.revoke_expired_tokens().await?;
    Ok(())
}
