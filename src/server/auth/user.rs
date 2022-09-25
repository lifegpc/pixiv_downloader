use super::super::preclude::*;
use crate::ext::json::ToJson2;
use crate::ext::try_err::{TryErr, TryErr3};
use crate::gettext;
use openssl::{hash::MessageDigest, pkcs5::pbkdf2_hmac};

const SALT: [u8; 64] = [
    14, 169, 19, 53, 220, 112, 183, 235, 112, 165, 131, 132, 68, 29, 167, 65, 150, 219, 121, 212,
    121, 47, 132, 195, 216, 119, 172, 134, 208, 11, 2, 80, 105, 176, 45, 194, 78, 84, 16, 169, 228,
    25, 195, 207, 144, 204, 171, 95, 8, 113, 93, 40, 41, 116, 80, 126, 253, 142, 245, 147, 148,
    136, 121, 220,
];

#[derive(Clone, Debug)]
/// Action to perform on a user.
pub enum AuthUserAction {
    /// Add a new user.
    Add,
}

pub struct AuthUserContext {
    ctx: Arc<ServerContext>,
    action: Option<AuthUserAction>,
    is_restful: bool,
}

impl AuthUserContext {
    pub fn new(ctx: Arc<ServerContext>, action: Option<AuthUserAction>, is_restful: bool) -> Self {
        Self {
            ctx,
            action,
            is_restful,
        }
    }

    async fn handle(&self, mut req: Request<Body>) -> JSONResult {
        let root_user = self
            .ctx
            .db
            .get_user(0)
            .await
            .try_err3(-1001, gettext("Failed to operate the database:"))?;
        let params = req
            .get_params()
            .await
            .try_err3(-1002, gettext("Failed to get parameters:"))?;
        let user = if root_user.is_some() {
            Some(
                self.ctx
                    .verify_token(&req, &params)
                    .await
                    .try_err3(-403, gettext("Failed to verify the token:"))?,
            )
        } else {
            None
        };
        match &self.action {
            Some(act) => match act {
                AuthUserAction::Add => {
                    if root_user.is_some() {
                        if !user.as_ref().expect("User not found:").is_admin {
                            return Err((9, gettext("Admin privileges required.")).into());
                        }
                    }
                    let name = params
                        .get("name")
                        .try_err((1, gettext("No user's name specified.")))?;
                    let username = params
                        .get("username")
                        .try_err((2, gettext("No username specified.")))?;
                    let password = params
                        .get("password")
                        .try_err((3, gettext("No password specified.")))?;
                    let password = base64::decode(password)
                        .try_err3(4, gettext("Failed to decode password with base64:"))?;
                    let rsa_key = self.ctx.rsa_key.lock().await;
                    match *rsa_key {
                        Some(ref key) => {
                            if key.is_too_old() {
                                return Err((
                                    6,
                                    gettext("RSA key is too old. A new key should be generated."),
                                )
                                    .into());
                            }
                            let password = key
                                .decrypt(&password)
                                .try_err3(7, gettext("Failed to decrypt password with RSA:"))?;
                            let mut hashed_password = [0; 64];
                            pbkdf2_hmac(
                                &password,
                                &SALT,
                                2048,
                                MessageDigest::sha512(),
                                &mut hashed_password,
                            )
                            .try_err3(11, gettext("Failed to hash password:"))?;
                            if root_user.is_none() {
                                let user = self
                                    .ctx
                                    .db
                                    .add_root_user(name, username, &hashed_password)
                                    .await
                                    .try_err3(8, gettext("Failed to add user to database:"))?;
                                Ok(user.to_json2())
                            } else {
                                let mut is_admin = params
                                    .get_bool("is_admin")
                                    .try_err3(
                                        9,
                                        &gettext("Failed to parse <opt>:")
                                            .replace("<opt>", "is_admin"),
                                    )?
                                    .unwrap_or(false);
                                let id = params.get_u64("id").try_err3(
                                    10,
                                    &gettext("Failed to parse <opt>:").replace("<opt>", "id"),
                                )?;
                                match id {
                                    Some(id) => {
                                        if id == 0 {
                                            is_admin = true;
                                        }
                                        let user = self
                                            .ctx
                                            .db
                                            .set_user(
                                                id,
                                                name,
                                                username,
                                                &hashed_password,
                                                is_admin,
                                            )
                                            .await
                                            .try_err3(
                                                12,
                                                gettext("Failed to set user in database:"),
                                            )?;
                                        Ok(user.to_json2())
                                    }
                                    None => Ok(json::object! {}),
                                }
                            }
                        }
                        None => return Err((5, gettext("No RSA key found.")).into()),
                    }
                }
            },
            None => {
                panic!("No action specified for AuthUserContext.");
            }
        }
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthUserContext {
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
        let builder = match &re {
            Ok(_) => builder,
            Err(err) => {
                if err.code <= -400 && err.code >= -600 {
                    builder.status((-err.code) as u16)
                } else if err.code < 0 {
                    builder.status(500)
                } else {
                    builder
                }
            }
        };
        Ok(builder.body(re.to_json2())?)
    }
}

pub struct AuthUserRoute {
    regex: Regex,
}

impl AuthUserRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+user(/+add)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Body> for AuthUserRoute {
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
                    return Some(Box::new(AuthUserContext::new(
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
                            "add" => Some(AuthUserAction::Add),
                            _ => return None,
                        }
                    }
                    None => {
                        if req.method() == Method::PUT {
                            Some(AuthUserAction::Add)
                        } else {
                            None
                        }
                    }
                };
                Some(Box::new(AuthUserContext::new(
                    Arc::clone(ctx),
                    action,
                    is_restful,
                )))
            }
            None => None,
        }
    }
}
