#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum FanboxAPIError {
    String(String),
}

impl From<&'static str> for FanboxAPIError {
    fn from(v: &'static str) -> Self {
        Self::String(v.to_owned())
    }
}
