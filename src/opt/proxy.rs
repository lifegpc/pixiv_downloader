use crate::ext::try_err::TryErr;
use crate::gettext;
use json::JsonValue;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;
use url::Url;

/// The error when parsing Proxy settings
#[derive(Debug, derive_more::From)]
pub enum ProxyError {
    /// String error
    String(String),
    /// Url parse error
    UrlParseError(url::ParseError),
}

impl Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_str(s.as_str()),
            Self::UrlParseError(s) => {
                f.write_str(gettext("Failed to parse URL:"))?;
                s.fmt(f)
            }
        }
    }
}

impl From<&str> for ProxyError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}

#[derive(Clone, Debug)]
/// Proxy settings
pub enum Proxy {
    /// Apply for all HTTP requests, [None] means do not proxy
    HTTP(Option<Url>),
    /// Apply for all HTTPS requests, [None] means do not proxy
    HTTPS(Option<Url>),
    /// Apply for all requests, [None] means do not proxy
    All(Option<Url>),
}

impl TryFrom<&JsonValue> for Proxy {
    type Error = ProxyError;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(s) => {
                return Ok(Self::All(Some(Url::parse(s)?)));
            }
            None => {}
        }
        if value.is_object() {
            let typ = value["type"]
                .as_str()
                .try_err(format!(
                    "{} {}",
                    gettext("Failed to get proxy's type:"),
                    value
                ))?
                .to_lowercase();
            let enable = value["enable"].as_bool().try_err(format!(
                "{} {}",
                gettext("Failed to get whether to enable proxy:"),
                value
            ))?;
            let proxy = if enable {
                Some(value["proxy"].as_str().try_err(format!(
                    "{} {}",
                    gettext("Failed to get proxy's proxy url:"),
                    value
                ))?)
            } else {
                None
            };
            if &typ == "all" {
                return Ok(Self::All(if enable {
                    Some(Url::parse(proxy.unwrap())?)
                } else {
                    None
                }));
            }
            if &typ == "http" {
                return Ok(Self::HTTP(if enable {
                    Some(Url::parse(proxy.unwrap())?)
                } else {
                    None
                }));
            }
            if &typ == "https" {
                return Ok(Self::HTTPS(if enable {
                    Some(Url::parse(proxy.unwrap())?)
                } else {
                    None
                }));
            }
        }
        Err(ProxyError::String(format!(
            "{} {}",
            gettext("Failed to parse proxy:"),
            value
        )))
    }
}

impl Proxy {
    /// Match the url.
    /// * `url` - Url
    pub fn r#match(&self, url: &Url) -> Option<Option<Url>> {
        match self {
            Self::All(d) => Some(d.clone()),
            Self::HTTP(d) => {
                let scheme = url.scheme();
                if scheme.is_empty() || scheme == "http" {
                    Some(d.clone())
                } else {
                    None
                }
            }
            Self::HTTPS(d) => {
                if url.scheme() == "https" {
                    Some(d.clone())
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
/// A list of [Proxy]
pub struct ProxyChain {
    /// Proxies
    proxies: Vec<Proxy>,
}

impl ProxyChain {
    /// Match the url
    /// * `url` - The url
    pub fn r#match(&self, url: &Url) -> Option<Url> {
        for i in self.proxies.iter() {
            match i.r#match(url) {
                Some(u) => {
                    return u;
                }
                None => {}
            }
        }
        None
    }
}

impl Default for ProxyChain {
    fn default() -> Self {
        Self {
            proxies: Vec::new(),
        }
    }
}

impl Deref for ProxyChain {
    type Target = Vec<Proxy>;
    fn deref(&self) -> &Self::Target {
        &self.proxies
    }
}

impl DerefMut for ProxyChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.proxies
    }
}

impl TryFrom<&JsonValue> for ProxyChain {
    type Error = ProxyError;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        let mut list = Vec::new();
        if value.is_array() {
            for i in value.members() {
                list.push(Proxy::try_from(i)?);
            }
            Ok(Self { proxies: list })
        } else {
            Err(ProxyError::from(gettext("Failed to parse proxy list.")))
        }
    }
}

impl TryFrom<JsonValue> for ProxyChain {
    type Error = ProxyError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

pub fn check_proxy(v: &JsonValue) -> bool {
    match ProxyChain::try_from(v) {
        Ok(_) => true,
        Err(e) => {
            log::error!("{}", e);
            false
        }
    }
}
