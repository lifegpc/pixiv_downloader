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
    /// Change a user's password.
    ChangePassword,
    /// Delete a user.
    Delete,
    /// Get a user's information.
    GetInfo,
    /// List users.
    List,
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
        if let Some(act) = self.action.as_ref() {
            if root_user.is_none() && !matches!(act, AuthUserAction::Add) {
                return Err((19, gettext("No root user, you need add a user first.")).into());
            }
        }
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
                AuthUserAction::ChangePassword => {
                    let token_id = match req.headers().get("X-TOKEN-ID") {
                        Some(s) => s.to_str().unwrap().to_owned(),
                        None => params.get("token_id").unwrap().to_owned(),
                    }
                    .parse::<u64>()
                    .unwrap();
                    let password = params
                        .get("password")
                        .try_err3(3, "No password specified.")?;
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
                            let user = self
                                .ctx
                                .db
                                .update_user_password(
                                    user.expect("User not found:").id,
                                    &hashed_password,
                                    token_id,
                                )
                                .await
                                .try_err3(8, gettext("Failed to update user in database:"))?;
                            Ok(user.to_json2())
                        }
                        None => Err((5, gettext("No RSA key found.")).into()),
                    }
                }
                AuthUserAction::Delete => {
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
                    let user = if let Some(id) = id {
                        self.ctx.db.get_user(id).await
                    } else if let Some(username) = username {
                        self.ctx.db.get_user_by_username(username).await
                    } else {
                        return Err((14, gettext("No user id or username specified.")).into());
                    }
                    .try_err3(-1001, gettext("Failed to operate the database:"))?
                    .try_err3(15, gettext("User not found."))?;
                    let re = self
                        .ctx
                        .db
                        .delete_user(user.id)
                        .await
                        .try_err3(-1001, gettext("Failed to operate the database:"))?;
                    Ok(json::JsonValue::Boolean(re))
                }
                AuthUserAction::GetInfo => {
                    let id = params.get_u64("id").try_err3(
                        13,
                        &gettext("Failed to parse <opt>:").replace("<opt>", "user id"),
                    )?;
                    let username = params.get("username");
                    let nuser = if let Some(id) = id {
                        self.ctx.db.get_user(id).await
                    } else if let Some(username) = username {
                        self.ctx.db.get_user_by_username(username).await
                    } else {
                        Ok(user.clone())
                    }
                    .try_err3(-1001, gettext("Failed to operate the database:"))?
                    .try_err3(15, gettext("User not found."))?;
                    let user = user.as_ref().expect("User not found:");
                    if root_user.is_some() && nuser.id != user.id {
                        if !user.is_admin {
                            return Err((9, gettext("Admin privileges required.")).into());
                        }
                    }
                    Ok(nuser.to_json2())
                }
                AuthUserAction::List => {
                    if root_user.is_some() {
                        if !user.as_ref().expect("User not found:").is_admin {
                            return Err((9, gettext("Admin privileges required.")).into());
                        }
                    }
                    let page = params
                        .get_u64_mult(&["page", "p"])
                        .try_err3(
                            20,
                            &gettext("Failed to parse <opt>:")
                                .replace("<opt>", gettext("page number")),
                        )?
                        .unwrap_or(1);
                    let page_count = params
                        .get_u64_mult(&["page_count", "pc"])
                        .try_err3(
                            22,
                            &gettext("Failed to parse <opt>:")
                                .replace("<opt>", gettext("page count")),
                        )?
                        .unwrap_or(10);
                    if page == 0 {
                        return Err((
                            21,
                            &gettext("<sth> should be greater than <num>.")
                                .replace("<sth>", gettext("Page number"))
                                .replace("<num>", "0"),
                        )
                            .into());
                    }
                    if page_count == 0 {
                        return Err((
                            23,
                            &gettext("<sth> should be greater than <num>.")
                                .replace("<sth>", gettext("Page count"))
                                .replace("<num>", "0"),
                        )
                            .into());
                    }
                    let id_only = params
                        .get_bool("id_only")
                        .try_err3(
                            24,
                            &gettext("Failed to parse <opt>:").replace("<opt>", "id_only"),
                        )?
                        .unwrap_or(false);
                    let offset = (page - 1) * page_count;
                    let data = if id_only {
                        let users = self
                            .ctx
                            .db
                            .list_users_id(offset, page_count)
                            .await
                            .try_err3(-1001, gettext("Failed to operate the database:"))?;
                        json::from(users)
                    } else {
                        let users = self
                            .ctx
                            .db
                            .list_users(offset, page_count)
                            .await
                            .try_err3(-1001, gettext("Failed to operate the database:"))?;
                        let mut tmp = Vec::with_capacity(users.len());
                        for user in users {
                            tmp.push(user.to_json2());
                        }
                        json::from(tmp)
                    };
                    Ok(json::object! { "page": page, "page_count": page_count, "data": data })
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
                PATCH,
                DELETE,
                GET,
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
            regex: Regex::new(
                r"^(/+api)?/+auth/+user(/+(add|update|delete|info|list|change/+(name|password)))?$",
            )
            .unwrap(),
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
                            "delete" => Some(AuthUserAction::Delete),
                            "info" => Some(AuthUserAction::GetInfo),
                            "list" => Some(AuthUserAction::List),
                            "update" => Some(AuthUserAction::Update),
                            _ => {
                                if m.starts_with("change/") {
                                    let m = m.trim_start_matches("change/");
                                    let m = m.trim_start_matches("/");
                                    match m {
                                        "name" => Some(AuthUserAction::ChangeName),
                                        "password" => Some(AuthUserAction::ChangePassword),
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
                        } else if m == Method::DELETE {
                            Some(AuthUserAction::Delete)
                        } else if m == Method::GET {
                            Some(AuthUserAction::GetInfo)
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
