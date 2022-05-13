use crate::ext::json::FromJson;
use crate::ext::json::ToJson;
use crate::gettext;
use crate::ext::try_err::TryErr;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum UseOrNot {
    /// Auto Detect
    Auto,
    /// Enabled
    Yes,
    /// Disabled
    No,
}

/// Convert to bool (whether to enable some features)
pub trait ToBool where Self: AsRef<UseOrNot> {
    /// Auto detect function.
    fn detect(&self) -> bool;
    /// Return whether to enable some features
    fn to_bool(&self) -> bool {
        match self.as_ref() {
            UseOrNot::Auto => { self.detect() }
            UseOrNot::Yes => { true }
            UseOrNot::No => { false }
        }
    }
}

impl AsRef<Self> for UseOrNot {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for UseOrNot {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<bool> for UseOrNot {
    fn from(v: bool) -> Self {
        if v {
            Self::Yes
        } else {
            Self::No
        }
    }
}

impl FromStr for UseOrNot {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        if s == "yes" || s == "true" {
            Ok(Self::Yes)
        } else if s == "no" || s == "false" {
            Ok(Self::No)
        } else if s == "auto" {
            Ok(Self::Auto)
        } else {
            Err(gettext("Failed to parse value."))
        }
    }
}

impl FromJson for UseOrNot {
    type Err = &'static str;
    fn from_json<T: ToJson>(v: T) -> Result<Self, Self::Err> {
        let v = v.to_json().try_err(gettext("Failed to get JSON object."))?;
        if v.is_boolean() {
            Ok(Self::from(v.as_bool().unwrap()))
        } else if v.is_string() {
            Self::from_json(v.as_str().unwrap())
        } else {
            Err(gettext("Unsupported JSON type."))
        }
    }
}
