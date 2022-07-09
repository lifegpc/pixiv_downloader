use crate::gettext;
use http::uri::InvalidUri;
use http::uri::Scheme;
use http::Uri;
use std::cmp::PartialEq;
use std::convert::From;
use std::convert::TryFrom;
use std::fmt::Display;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
/// Other host
pub struct CorsEntry {
    host: String,
    port: u16,
    scheme: Scheme,
    include_subdomain: bool,
}

impl CorsEntry {
    pub fn new(url: Uri) -> Result<Self, &'static str> {
        match url.host() {
            Some(host) => {
                let include_subdomain = host.starts_with(".");
                let host = if include_subdomain {
                    host.trim_start_matches(".")
                } else {
                    host
                };
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
                    include_subdomain,
                })
            }
            None => Err(gettext("hostname not found")),
        }
    }
}

impl From<&SocketAddr> for CorsEntry {
    fn from(v: &SocketAddr) -> Self {
        Self {
            host: v.ip().to_string(),
            port: v.port(),
            scheme: Scheme::HTTP,
            include_subdomain: false,
        }
    }
}

impl From<SocketAddr> for CorsEntry {
    fn from(v: SocketAddr) -> Self {
        Self::from(&v)
    }
}

impl FromStr for CorsEntry {
    type Err = CorsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<Uri> for CorsEntry {
    type Error = &'static str;
    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl PartialEq<Uri> for CorsEntry {
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
                if host == &self.host && scheme == self.scheme && port == self.port {
                    return true;
                }
                if self.include_subdomain {
                    let host2 = format!(".{}", self.host);
                    host.ends_with(&host2) && scheme == self.scheme && port == self.port
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

impl PartialEq<CorsEntry> for Uri {
    fn eq(&self, other: &CorsEntry) -> bool {
        other == self
    }
}

impl PartialEq<str> for CorsEntry {
    fn eq(&self, other: &str) -> bool {
        match Uri::from_str(other) {
            Ok(uri) => *self == uri,
            Err(_) => false,
        }
    }
}

impl PartialEq<&str> for CorsEntry {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<CorsEntry> for str {
    fn eq(&self, other: &CorsEntry) -> bool {
        other == self
    }
}

impl PartialEq<CorsEntry> for &str {
    fn eq(&self, other: &CorsEntry) -> bool {
        other == self
    }
}

impl TryFrom<&str> for CorsEntry {
    type Error = CorsError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(Uri::from_str(value)?)?)
    }
}

impl TryFrom<String> for CorsEntry {
    type Error = CorsError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

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

impl From<&SocketAddr> for CorsHost {
    fn from(v: &SocketAddr) -> Self {
        Self {
            host: v.ip().to_string(),
            port: v.port(),
            scheme: Scheme::HTTP,
        }
    }
}

impl From<SocketAddr> for CorsHost {
    fn from(v: SocketAddr) -> Self {
        Self::from(&v)
    }
}

impl FromStr for CorsHost {
    type Err = CorsError;
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
    type Error = CorsError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(Uri::from_str(value)?)?)
    }
}

impl TryFrom<String> for CorsHost {
    type Error = CorsError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum CorsError {
    String(String),
    InvaildUri(InvalidUri),
}

impl From<&str> for CorsError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}

#[test]
fn test_cors_entry() {
    let ent = CorsEntry::try_from("https://.test.com").unwrap();
    let uri = Uri::from_str("https://test.com").unwrap();
    assert!(ent == uri);
    assert!(uri == ent);
    assert!(ent == "https://a.test.com");
    assert!(ent != "http://test.com");
    assert!(ent == "https://a.test.com/test/a/s");
    let ent2 = CorsEntry::try_from("https://test.com").unwrap();
    assert!(ent2 == "https://test.com");
    assert!(ent2 != "https://a.test.com");
    assert!(ent2 == "https://test.com/et/sd");
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
    assert!(host == CorsHost::from(SocketAddr::from_str("127.0.0.1:8080").unwrap()));
}
