use crate::ext::json::ToJson2;
use crate::gettext;
use json::JsonValue;

#[derive(Clone, Debug)]
/// Error information of a request
pub struct JSONError {
    /// Error code
    pub code: i32,
    /// Error message
    pub msg: String,
    /// The debug information of the error
    pub debug_msg: Option<JsonValue>,
}

impl From<(i32, String)> for JSONError {
    fn from((code, msg): (i32, String)) -> Self {
        Self {
            code,
            msg,
            debug_msg: None,
        }
    }
}

impl From<(i32, String, Option<JsonValue>)> for JSONError {
    fn from((code, msg, debug_msg): (i32, String, Option<JsonValue>)) -> Self {
        Self {
            code,
            msg,
            debug_msg,
        }
    }
}

impl From<crate::db::PixivDownloaderDbError> for JSONError {
    fn from(e: crate::db::PixivDownloaderDbError) -> Self {
        Self {
            code: -1001,
            msg: format!("{} {}", gettext("Failed to operate the database:"), e),
            debug_msg: Some(format!("{:?}", e).to_json2()),
        }
    }
}

impl From<crate::error::PixivDownloaderError> for JSONError {
    fn from(e: crate::error::PixivDownloaderError) -> Self {
        Self {
            code: -500,
            msg: format!("{}", e),
            debug_msg: Some(format!("{:?}", e).to_json2()),
        }
    }
}

pub type JSONResult = Result<JsonValue, JSONError>;

impl ToJson2 for JSONResult {
    fn to_json2(&self) -> JsonValue {
        match self {
            Self::Ok(v) => json::object! {
                "ok": true,
                "code": 0,
                "result": v.clone(),
            },
            Self::Err(e) => json::object! {
                "ok": false,
                "code": e.code,
                "msg": e.msg.as_str(),
                "debug_msg": e.debug_msg.clone().unwrap_or(JsonValue::Null),
            },
        }
    }
}
