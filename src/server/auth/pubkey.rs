use super::super::preclude::*;
use crate::ext::{json::ToJson2, try_err::TryErr3};
use crate::gettext;
use bytes::BytesMut;
use chrono::{DateTime, Utc};
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

pub struct RSAKey {
    pub key: Rsa<Private>,
    pub generated_time: DateTime<Utc>,
}

impl RSAKey {
    pub fn new() -> Result<Self, openssl::error::ErrorStack> {
        Ok(Self {
            key: Rsa::generate(4096)?,
            generated_time: Utc::now(),
        })
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<BytesMut, openssl::error::ErrorStack> {
        let tosize = self.key.size() as usize;
        let mut buf = BytesMut::with_capacity(tosize);
        buf.resize(tosize, 0);
        let real = self.key.private_decrypt(&data, &mut buf, Padding::PKCS1)?;
        buf.truncate(real);
        Ok(buf)
    }

    #[cfg(test)]
    pub fn encrypt(&self, data: &[u8]) -> Result<BytesMut, openssl::error::ErrorStack> {
        let tosize = self.key.size() as usize;
        let mut buf = BytesMut::with_capacity(tosize);
        buf.resize(tosize, 0);
        self.key.public_encrypt(&data, &mut buf, Padding::PKCS1)?;
        Ok(buf)
    }

    pub fn is_too_old(&self) -> bool {
        self.generated_time < Utc::now() - chrono::Duration::hours(1)
    }
}

pub struct AuthPubkeyContext {
    ctx: Arc<ServerContext>,
}

impl AuthPubkeyContext {
    pub fn new(ctx: Arc<ServerContext>) -> Self {
        Self { ctx }
    }

    async fn handle(&self) -> JSONResult {
        let mut rsa_key = self.ctx.rsa_key.lock().await;
        match &*rsa_key {
            Some(key) => {
                if key.is_too_old() {
                    *rsa_key =
                        Some(RSAKey::new().try_err3(1, gettext("Failed to generate RSA key:"))?);
                }
            }
            None => {
                *rsa_key = Some(RSAKey::new().try_err3(1, gettext("Failed to generate RSA key:"))?);
            }
        }
        let rsa_key = rsa_key.as_ref().unwrap();
        let key = rsa_key.key.public_key_to_pem().try_err3(2, gettext("Failed to serializes the public key into a PEM-encoded SubjectPublicKeyInfo structure:"))?;
        Ok(json::object! {
            "key": String::from_utf8(key).try_err3(3, gettext("Failed to encode pem with UTF-8:"))?,
            "generated_time": rsa_key.generated_time.timestamp(),
        })
    }
}

#[async_trait]
impl ResponseJsonFor<Body> for AuthPubkeyContext {
    async fn response_json(
        &self,
        req: Request<Body>,
    ) -> Result<Response<JsonValue>, PixivDownloaderError> {
        filter_http_methods!(
            req,
            json::object! {},
            true,
            self.ctx,
            allow_headers = [CONTENT_TYPE],
            GET,
            OPTIONS,
            POST,
        );
        Ok(builder.body(self.handle().await.to_json2())?)
    }
}

pub struct AuthPubkeyRoute {
    regex: Regex,
}

impl AuthPubkeyRoute {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(r"^(/+api)?/+auth/+pubkey(/.*)?$").unwrap(),
        }
    }
}

impl MatchRoute<Body, Pin<Box<HttpBodyType>>> for AuthPubkeyRoute {
    fn match_route(
        &self,
        ctx: &Arc<ServerContext>,
        req: &http::Request<Body>,
    ) -> Option<Box<ResponseForType>> {
        if self.regex.is_match(req.uri().path()) {
            Some(Box::new(AuthPubkeyContext::new(Arc::clone(ctx))))
        } else {
            None
        }
    }
}

#[test]
fn test_rsa_decrypt() {
    let key = RSAKey::new().unwrap();
    let data = b"Hello, world!";
    let enc = key.encrypt(data).unwrap();
    let dec = key.decrypt(&enc).unwrap();
    assert_eq!(data, &dec[..]);
    let mut data = BytesMut::with_capacity(256);
    data.resize(256, 0);
    openssl::rand::rand_bytes(&mut data).unwrap();
    let enc = key.encrypt(&data).unwrap();
    let dec = key.decrypt(&enc).unwrap();
    assert_eq!(data, dec);
}
