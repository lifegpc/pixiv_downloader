use super::check::CheckUnknown;
use crate::error::PixivDownloaderError;
use json::JsonValue;
use proc_macros::check_json_keys;
use wreq::{Response, StatusCode};

#[derive(Debug)]
pub struct PixivAppHTTPError {
    status: StatusCode,
}

impl PixivAppHTTPError {
    pub fn new(status: StatusCode) -> Self {
        Self { status }
    }
}

impl std::fmt::Display for PixivAppHTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.status.as_u16(),
            self.status.canonical_reason().unwrap_or("")
        )
    }
}

pub struct PixivAppAPIError {
    data: JsonValue,
    status: StatusCode,
}

impl CheckUnknown for PixivAppAPIError {
    fn check_unknown(&self) -> Result<(), String> {
        check_json_keys!(
            "user_message"+,
            "message"+,
            "reason"+,
            "user_message_details"
        );
        Ok(())
    }
}

impl PixivAppAPIError {
    pub fn new(data: JsonValue, status: StatusCode) -> Self {
        Self { data, status }
    }

    pub fn user_message(&self) -> Option<&str> {
        self.data["user_message"].as_str()
    }

    pub fn message(&self) -> Option<&str> {
        self.data["message"].as_str()
    }

    pub fn reason(&self) -> Option<&str> {
        self.data["reason"].as_str()
    }

    pub fn user_message_details(&self) -> &JsonValue {
        &self.data["user_message_details"]
    }
}

impl std::error::Error for PixivAppAPIError {}

impl std::fmt::Debug for PixivAppAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixivAppAPIError")
            .field("user_message", &self.user_message())
            .field("message", &self.message())
            .field("reason", &self.reason())
            .field("user_message_details", &self.user_message_details())
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for PixivAppAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user_message) = self.user_message() {
            write!(
                f,
                "{} ({} {})",
                user_message,
                self.status.as_u16(),
                self.status.canonical_reason().unwrap_or("")
            )
        } else if let Some(message) = self.message() {
            write!(f, "{} (", message)?;
            if let Some(reason) = self.reason() {
                write!(f, "{}, ", reason)?;
            }
            write!(
                f,
                "{} {})",
                self.status.as_u16(),
                self.status.canonical_reason().unwrap_or("")
            )
        } else {
            write!(
                f,
                "{} {}: ",
                self.status.as_u16(),
                self.status.canonical_reason().unwrap_or("")
            )?;
            write!(f, "{}", self.data.pretty(2).as_str())
        }
    }
}

#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum PixivAppError {
    HTTP(PixivAppHTTPError),
    API(PixivAppAPIError),
}

impl PixivAppError {
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::HTTP(err) => err.status.as_u16() == 404,
            Self::API(err) => err.status.as_u16() == 404,
        }
    }
}

pub async fn handle_error(res: Response) -> Result<JsonValue, PixivDownloaderError> {
    let status = res.status();
    if let Ok(obj) = json::parse(&res.text().await?) {
        let err = &obj["error"];
        if status.is_success() && err.is_null() {
            Ok(obj)
        } else if !err.is_null() {
            Err(PixivDownloaderError::from(PixivAppError::from(
                PixivAppAPIError::new(err.clone(), status),
            )))
        } else {
            Err(PixivDownloaderError::from(PixivAppError::from(
                PixivAppHTTPError::new(status),
            )))
        }
    } else {
        Err(PixivDownloaderError::from(PixivAppError::from(
            PixivAppHTTPError::new(status),
        )))
    }
}
