use super::super::preclude::*;
use super::{PASSWORD_ITER, PASSWORD_SALT};
use crate::ext::json::ToJson2;
use crate::ext::try_err::{TryErr, TryErr3};
use crate::gettext;
use bytes::BytesMut;
use openssl::{hash::MessageDigest, pkcs5::pbkdf2_hmac};

#[derive(Clone, Debug)]
/// Action to perform on a user.
pub enum AuthUserAction {
    /// Add a new user.
    Add,
    /// Change a user's name.
    ChangeName,
    /// Update a existed user.
    Update,
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
                                &PASSWORD_SALT,
                                PASSWORD_ITER,
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
                                    None => {
                                        let user = self
                                            .ctx
                                            .db
                                            .add_user(name, username, &hashed_password, is_admin)
                                            .await
                                            .try_err3(
                                                12,
                                                gettext("Failed to add user to database:"),
                                            )?;
                                        Ok(user.to_json2())
                                    }
                                }
                            }
                        }
                        None => return Err((5, gettext("No RSA key found.")).into()),
                    }
                }
                AuthUserAction::ChangeName => {
                    let name = params.get("name").try_err3(18, "No name specified.")?;
                    let user = self
                        .ctx
                        .db
                        .update_user_name(user.expect("User not found.").id, name)
                        .await
                        .try_err3(-1001, gettext("Failed to operate the database:"))?;
                    Ok(user.to_json2())
                }
                AuthUserAction::Update => {
                    if root_user.is_some() {
                        if !user.as_ref().expect("User not found:").is_admin {
                            return Err((9, gettext("Admin privileges required.")).into());
                        }
                    }
                    let id = params.get_u64("id").try_err3(
                        13,
                        &gettext("Failed to parse <opt>:").replace("<opt>", "user id"),
                    )?;
                    let username = params.get("username");
                    let mut user = if let Some(id) = id {
                        self.ctx.db.get_user(id).await
                    } else if let Some(username) = username {
                        self.ctx.db.get_user_by_username(username).await
                    } else {
                        return Err((14, gettext("No user id or username specified.")).into());
                    }
                    .try_err3(-1001, gettext("Failed to operate the database:"))?
                    .ok_or((15, gettext("User not found.")))?;
                    if let Some(username) = username {
                        if username != user.username {
                            user.username = username.to_owned();
                        }
                    }
                    if let Some(name) = params.get("name") {
                        if name != user.name {
                            user.name = name.to_owned();
                        }
                    }
                    if let Some(password) = params.get("password") {
                        let password = base64::decode(password)
                            .try_err3(4, gettext("Failed to decode password with base64:"))?;
                        let rsa_key = self.ctx.rsa_key.lock().await;
                        match *rsa_key {
                            Some(ref key) => {
                                if key.is_too_old() {
                                    return Err((
                                        6,
                                        gettext(
                                            "RSA key is too old. A new key should be generated.",
                                        ),
                                    )
                                        .into());
                                }
                                let password = key
                                    .decrypt(&password)
                                    .try_err3(7, gettext("Failed to decrypt password with RSA:"))?;
                                let mut hashed_password = [0; 64];
                                pbkdf2_hmac(
                                    &password,
                                    &PASSWORD_SALT,
                                    PASSWORD_ITER,
                                    MessageDigest::sha512(),
                                    &mut hashed_password,
                                )
                                .try_err3(11, gettext("Failed to hash password:"))?;
                                let pw: &[u8] = &user.password;
                                if &hashed_password != pw {
                                    let pw: &[u8] = &hashed_password;
                                    user.password = BytesMut::from(pw);
                                }
                            }
                            None => return Err((5, gettext("No RSA key found.")).into()),
                        }
                    }
                    if let Some(is_admin) = params.get_bool("is_admin").try_err3(
                        16,
                        &gettext("Failed to parse <opt>:").replace("<opt>", "is_admin"),
                    )? {
                        if user.id == 0 && !is_admin {
                            return Err((
                                17,
                                gettext("Cannot change admin privileges of root user."),
                            )
                                .into());
                        }
                        user.is_admin = is_admin;
                    }
                    let user = self
                        .ctx
                        .db
                        .update_user(
                            user.id,
                            &user.name,
                            &user.username,
                            &user.password,
                            user.is_admin,
                        )
                        .await
                        .try_err3(-1001, gettext("Failed to operate the database:"))?;
                    Ok(user.to_json2())
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
        self.ctx.response_json_result(builder, re)
    }
}

pub struct AuthUserRoute {
    regex: Regex,
}

impl AuthUserRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+user(/+(add|update|change/+name))?$").unwrap(),
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
                            "update" => Some(AuthUserAction::Update),
                            _ => {
                                if m.starts_with("change/") {
                                    let m = m.trim_start_matches("change/");
                                    let m = m.trim_start_matches("/");
                                    match m {
                                        "name" => Some(AuthUserAction::ChangeName),
                                        _ => return None,
                                    }
                                } else {
                                    return None;
                                }
                            }
                        }
                    }
                    None => {
                        let m = req.method();
                        if m == Method::PUT {
                            Some(AuthUserAction::Add)
                        } else if m == Method::PATCH {
                            Some(AuthUserAction::Update)
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
