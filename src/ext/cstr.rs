#[cfg(feature = "c_fixed_string")]
use c_fixed_string::CFixedStr;
#[cfg(feature = "c_fixed_string")]
use c_fixed_string::CFixedString;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::NulError;
use std::fmt::Display;

#[derive(Debug, derive_more::From, PartialEq)]
pub enum ToCStrError {
    Null(NulError),
}

impl Display for ToCStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null(e) => e.fmt(f),
        }
    }
}

pub trait ToCStr {
    fn to_cstr(&self) -> Result<CString, ToCStrError>;
}

impl ToCStr for CString {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        Ok(self.clone())
    }
}

impl ToCStr for CStr {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        Ok(self.to_owned())
    }
}

impl ToCStr for [u8] {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        Ok(CString::new(self)?)
    }
}

impl ToCStr for &str {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        (*self).as_bytes().to_cstr()
    }
}

impl ToCStr for String {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        self.as_bytes().to_cstr()
    }
}

#[cfg(feature = "c_fixed_string")]
impl ToCStr for CFixedStr {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        Ok(self.to_c_str().into_owned())
    }
}

#[cfg(feature = "c_fixed_string")]
impl ToCStr for CFixedString {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        Ok(self.to_c_str().into_owned())
    }
}

impl<'a, T: ToCStr> ToCStr for &'a T {
    fn to_cstr(&self) -> Result<CString, ToCStrError> {
        (*self).to_cstr()
    }
}
