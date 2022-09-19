use json::JsonValue;

use crate::ext::json::ToJson2;

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
