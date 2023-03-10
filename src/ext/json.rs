use json::JsonValue;
use std::ops::Deref;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

pub trait ToJson {
    fn to_json(&self) -> Option<JsonValue>;
}

pub trait ToJson2 {
    fn to_json2(&self) -> JsonValue;
}

impl ToJson for &str {
    fn to_json(&self) -> Option<JsonValue> {
        Some(JsonValue::String(String::from(*self)))
    }
}

impl ToJson2 for &str {
    fn to_json2(&self) -> JsonValue {
        JsonValue::String(String::from(*self))
    }
}

impl ToJson for String {
    fn to_json(&self) -> Option<JsonValue> {
        Some(JsonValue::String(self.to_string()))
    }
}

impl ToJson2 for String {
    fn to_json2(&self) -> JsonValue {
        JsonValue::String(self.to_string())
    }
}

impl ToJson for JsonValue {
    fn to_json(&self) -> Option<JsonValue> {
        Some(self.clone())
    }
}

impl ToJson2 for JsonValue {
    fn to_json2(&self) -> JsonValue {
        self.clone()
    }
}

impl<T: ToJson> ToJson for &T {
    fn to_json(&self) -> Option<JsonValue> {
        (*self).to_json()
    }
}

impl<T: ToJson2> ToJson2 for &T {
    fn to_json2(&self) -> JsonValue {
        (*self).to_json2()
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Option<JsonValue> {
        match self {
            Some(d) => d.to_json(),
            None => None,
        }
    }
}

impl<T: ToJson> ToJson for RwLockReadGuard<'_, T> {
    fn to_json(&self) -> Option<JsonValue> {
        self.deref().to_json()
    }
}

impl<T: ToJson2> ToJson2 for RwLockReadGuard<'_, T> {
    fn to_json2(&self) -> JsonValue {
        self.deref().to_json2()
    }
}

impl<T: ToJson> ToJson for RwLockWriteGuard<'_, T> {
    fn to_json(&self) -> Option<JsonValue> {
        self.deref().to_json()
    }
}

impl<T: ToJson2> ToJson2 for RwLockWriteGuard<'_, T> {
    fn to_json2(&self) -> JsonValue {
        self.deref().to_json2()
    }
}

impl ToJson for bool {
    #[inline]
    fn to_json(&self) -> Option<JsonValue> {
        Some(JsonValue::from(self.clone()))
    }
}

impl ToJson2 for bool {
    #[inline]
    fn to_json2(&self) -> JsonValue {
        JsonValue::from(self.clone())
    }
}

macro_rules! impl_tojson_for_int {
    ($int:ty) => {
        impl ToJson for $int {
            #[inline]
            fn to_json(&self) -> Option<JsonValue> {
                Some(JsonValue::from(self.clone()))
            }
        }
        impl ToJson2 for $int {
            #[inline]
            fn to_json2(&self) -> JsonValue {
                JsonValue::from(self.clone())
            }
        }
    };
}

impl_tojson_for_int!(i8);
impl_tojson_for_int!(i16);
impl_tojson_for_int!(i32);
impl_tojson_for_int!(i64);
impl_tojson_for_int!(u8);
impl_tojson_for_int!(u16);
impl_tojson_for_int!(u32);
impl_tojson_for_int!(u64);

pub trait FromJson
where
    Self: Sized,
{
    type Err;
    fn from_json<T: ToJson>(v: T) -> Result<Self, Self::Err>;
}
