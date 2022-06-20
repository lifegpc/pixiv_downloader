use crate::ext::try_err::TryErr;
use crate::gettext;
use json::JsonValue;
use std::convert::TryFrom;
use url::Url;

#[derive(Debug, derive_more::From)]
pub enum ProxyError {
    String(String),
    UrlParseError(url::ParseError),
}

impl From<&str> for ProxyError {
    fn from(v: &str) -> Self {
        Self::String(String::from(v))
    }
}

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
