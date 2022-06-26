use json::JsonValue;

pub fn parse_u64(data: &JsonValue) -> Option<u64> {
    match data.as_u64() {
        Some(num) => Some(num),
        None => match data.as_str() {
            Some(num) => match num.trim().parse::<u64>() {
                Ok(num) => Some(num),
                Err(_) => None,
            },
            None => None,
        },
    }
}
