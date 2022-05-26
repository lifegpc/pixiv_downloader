use super::part_status::OutOfBoundsError;
use crate::gettext;
use int_enum::IntEnum;
use int_enum::IntEnumError;
use std::convert::From;
use std::fmt::Display;
use std::string::FromUtf8Error;

/// Pd file's error
#[derive(Debug, derive_more::From)]
pub enum PdFileError {
    IoError(std::io::Error),
    String(String),
    InvalidPdFile,
    Unsupported,
    Utf8Error(FromUtf8Error),
}

impl Display for PdFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => {
                f.write_str(gettext("Errors occured when operating files: "))?;
                e.fmt(f)?;
            }
            Self::String(e) => {
                f.write_str(e)?;
            }
            Self::InvalidPdFile => {
                f.write_str(gettext("Invalid pd file."))?;
            }
            Self::Unsupported => {
                f.write_str(gettext("The pd file is newer version, please update the program."))?;
            }
            Self::Utf8Error(e) => {
                f.write_str(gettext("Failed to decode UTF-8: "))?;
                e.fmt(f)?;
            }
        }
        Ok(())
    }
}

impl From<&str> for PdFileError {
    fn from(value: &str) -> Self {
        PdFileError::String(String::from(value))
    }
}

impl<T: IntEnum> From<IntEnumError<T>> for PdFileError {
    fn from(e: IntEnumError<T>) -> Self {
        Self::String(format!("{} {}", gettext("Invalid pd file: "), e))
    }
}

impl<T: Display> From<OutOfBoundsError<T>> for PdFileError {
    fn from(e: OutOfBoundsError<T>) -> Self {
        Self::String(format!("{}", e))
    }
}
