use json::JsonValue;

pub trait ToJson {
    fn to_json(&self) -> Option<JsonValue>;
}

impl ToJson for &str {
    fn to_json(&self) -> Option<JsonValue> {
        Some(JsonValue::String(String::from(*self)))
    }
}

impl ToJson for String {
    fn to_json(&self) -> Option<JsonValue> {
        Some(JsonValue::String(self.to_string()))
    }
}

impl ToJson for JsonValue {
    fn to_json(&self) -> Option<JsonValue> {
        Some(self.clone())
    }
}

impl<T: ToJson> ToJson for &T {
    fn to_json(&self) -> Option<JsonValue> {
        (*self).to_json()
    }
}

pub trait FromJson where Self: Sized {
    type Err;
    fn from_json<T: ToJson>(v: T) -> Result<Self, Self::Err>;
}
