use crate::gettext;
use http::uri::InvalidUri;
use http::uri::Scheme;
use http::Uri;
use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
/// Current host
pub struct CorsHost {
    host: String,
    port: u16,
    scheme: Scheme,
}

impl CorsHost {
    pub fn new(url: Uri) -> Result<Self, &'static str> {
        match url.host() {
            Some(host) => {
                let scheme = match url.scheme() {
                    Some(scheme) => scheme.to_owned(),
                    None => Scheme::HTTP,
                };
                let port = match url.port() {
                    Some(port) => port.as_u16(),
                    None => match scheme.as_str() {
                        "http" => 80,
                        "https" => 443,
                        _ => {
                            return Err(gettext("port not found."));
                        }
                    },
                };
                Ok(Self {
                    host: host.to_owned(),
                    port,
                    scheme,
                })
            }
            None => Err(gettext("hostname not found.")),
        }
    }
}

impl Display for CorsHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}://{}", self.scheme, self.port))
    }
}

impl FromStr for CorsHost {
    type Err = CorsHostError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl PartialEq<Uri> for CorsHost {
    fn eq(&self, other: &Uri) -> bool {
        match other.host() {
            Some(host) => {
                let scheme = match other.scheme() {
                    Some(scheme) => scheme.to_owned(),
                    None => Scheme::HTTP,
                };
                let port = match other.port() {
                    Some(port) => port.as_u16(),
                    None => match scheme.as_str() {
                        "http" => 80,
                        "https" => 443,
                        _ => {
                            return false;
                        }
                    },
                };
                host == &self.host && scheme == self.scheme && port == self.port
            }
            None => false,
        }
    }
}

impl PartialEq<CorsHost> for Uri {
    fn eq(&self, other: &CorsHost) -> bool {
        other == self
    }
}

impl PartialEq<str> for CorsHost {
    fn eq(&self, other: &str) -> bool {
        match Uri::from_str(other) {
            Ok(uri) => *self == uri,
            Err(_) => false,
        }
    }
}

impl PartialEq<&str> for CorsHost {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<CorsHost> for str {
    fn eq(&self, other: &CorsHost) -> bool {
        other == self
    }
}

impl PartialEq<CorsHost> for &str {
    fn eq(&self, other: &CorsHost) -> bool {
        other == self
    }
}

impl TryFrom<Uri> for CorsHost {
    type Error = &'static str;
    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for CorsHost {
    type Error = CorsHostError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(Uri::from_str(value)?)?)
    }
}

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum CorsHostError {
    String(String),
    InvaildUri(InvalidUri),
}

impl From<&str> for CorsHostError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}

#[test]
fn test_cors_host() {
    let host = CorsHost::try_from("127.0.0.1:8080").unwrap();
    let uri = Uri::from_str("http://127.0.0.1:8080").unwrap();
    assert!(host == uri);
    assert!(uri == host);
    assert!(host == Uri::from_str("127.0.0.1:8080").unwrap());
    assert!(host != Uri::from_str("127.0.0.1").unwrap());
    assert!(host != Uri::from_str("https://127.0.0.1:8080").unwrap());
    assert!(host == Uri::from_str("http://test:password@127.0.0.1:8080/test").unwrap());
    assert!(host != Uri::from_str("http://test:password@127.0.0.2:8080/test").unwrap());
    let host2 = CorsHost::try_from("127.0.0.1").unwrap();
    assert!(host2 == Uri::from_str("127.0.0.1").unwrap());
    assert!(host2 == Uri::from_str("127.0.0.1:80").unwrap());
    assert!(host2 != Uri::from_str("https://127.0.0.1").unwrap());
    assert!(host2 == "127.0.0.1");
    assert!("127.0.0.1" == host2);
    let host3 = CorsHost::try_from("https://127.0.0.1").unwrap();
    assert!(host3 == Uri::from_str("https://127.0.0.1").unwrap());
    assert!(host3 == Uri::from_str("https://127.0.0.1:443").unwrap());
    assert!(host3 != Uri::from_str("http://127.0.0.1").unwrap());
    assert!(host != host3);
    assert!(host == CorsHost::try_from("http://127.0.0.1:8080").unwrap());
}
