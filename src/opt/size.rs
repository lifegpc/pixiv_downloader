use json::JsonValue;
use std::convert::TryFrom;

/// Parse file size
pub fn parse_size(obj: &JsonValue) -> Option<u64> {
    match obj.as_u64() {
        Some(s) => Some(s),
        None => match obj.as_str() {
            Some(s) => match parse_size::parse_size(s) {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            None => None,
        },
    }
}

/// Parse file size as [i32]
pub fn parse_u32_size(obj: &JsonValue) -> Option<u32> {
    match parse_size(obj) {
        Some(s) => match u32::try_from(s) {
            Ok(s) => Some(s),
            Err(_) => None,
        },
        None => None,
    }
}
