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

impl<S> From<(i32, &S)> for JSONError
where
    S: AsRef<str> + ?Sized,
{
    fn from((code, msg): (i32, &S)) -> Self {
        Self {
            code,
            msg: msg.as_ref().to_owned(),
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

impl<S> From<(i32, &S, Option<JsonValue>)> for JSONError
where
    S: AsRef<str> + ?Sized,
{
    fn from((code, msg, debug_msg): (i32, &S, Option<JsonValue>)) -> Self {
        Self {
            code,
            msg: msg.as_ref().to_owned(),
            debug_msg,
        }
    }
}

impl<S, V> From<(i32, &S, &V)> for JSONError
where
    S: AsRef<str> + ?Sized,
    V: AsRef<str> + ?Sized,
{
    fn from((code, msg, debug_msg): (i32, &S, &V)) -> Self {
        Self {
            code,
            msg: msg.as_ref().to_owned(),
            debug_msg: Some(debug_msg.as_ref().to_json2()),
        }
    }
}

impl From<(i32, String, String)> for JSONError {
    fn from((code, msg, debug_msg): (i32, String, String)) -> Self {
        Self {
            code,
            msg,
            debug_msg: Some(debug_msg.to_json2()),
        }
    }
}

impl From<crate::db::PixivDownloaderDbError> for JSONError {
    fn from(e: crate::db::PixivDownloaderDbError) -> Self {
        Self::from((
            -1001,
            format!("{} {}", gettext("Failed to operate the database:"), e),
            format!("{:?}", e),
        ))
    }
}

impl From<crate::error::PixivDownloaderError> for JSONError {
    fn from(e: crate::error::PixivDownloaderError) -> Self {
        Self::from((-500, format!("{}", e), format!("{:?}", e)))
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
