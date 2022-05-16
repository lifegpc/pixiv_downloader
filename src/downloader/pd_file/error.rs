use crate::gettext;
use std::convert::From;
use std::fmt::Display;

#[derive(Debug, derive_more::From)]
pub enum PdFileError {
    IoError(std::io::Error),
    String(String),
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
        }
        Ok(())
    }
}

impl From<&str> for PdFileError {
    fn from(value: &str) -> Self {
        PdFileError::String(String::from(value))
    }
}
