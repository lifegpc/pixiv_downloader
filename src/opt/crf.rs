use json::JsonValue;

pub fn check_crf(obj: &JsonValue) -> bool {
    match obj.as_f32() {
        Some(crf) => crf >= -1f32,
        None => false,
    }
}
