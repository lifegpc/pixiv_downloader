use super::super::preclude::*;
use crate::ext::{json::ToJson2, try_err::TryErr3};
use crate::gettext;
use chrono::{DateTime, Utc};
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

pub struct RSAKey {
    pub key: RsaPrivateKey,
    pub generated_time: DateTime<Utc>,
}

impl RSAKey {
    pub fn new() -> Result<Self, rsa::Error> {
        let mut rng = rand::thread_rng(); // rand@0.8
        let bits = 4096;
        let key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Ok(Self {
            key,
            generated_time: Utc::now(),
        })
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, rsa::Error> {
        let buf = self.key.decrypt(Pkcs1v15Encrypt, data)?;
        Ok(buf)
    }

    #[cfg(test)]
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, rsa::Error> {
        let mut rng = rand::thread_rng();
        let buf = self.key.to_public_key().encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
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
        let key = rsa_key
            .key
            .to_public_key()
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .try_err3(2, gettext("Failed to serialize the public key into PEM format:"))?;
        Ok(json::object! {
            "key": key,
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
    use rand::prelude::*;
    use bytes::BytesMut;
    let key = RSAKey::new().unwrap();
    let data = b"Hello, world!";
    let enc = key.encrypt(data).unwrap();
    let dec = key.decrypt(&enc).unwrap();
    assert_eq!(data, &dec[..]);
    let mut data = BytesMut::with_capacity(256);
    data.resize(256, 0);
    let mut rng = rand::thread_rng();
    data.shuffle(&mut rng);
    let enc = key.encrypt(&data).unwrap();
    let dec = key.decrypt(&enc).unwrap();
    assert_eq!(data, dec);
}
