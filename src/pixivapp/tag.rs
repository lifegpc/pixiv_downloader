use super::check::CheckUnknown;
use json::JsonValue;
use proc_macros::check_json_keys;

pub struct Tag {
    data: JsonValue,
}

impl Tag {
    pub fn new(data: JsonValue) -> Self {
        Self { data }
    }

    pub fn name(&self) -> Option<&str> {
        self.data["name"].as_str()
    }

    pub fn translated_name(&self) -> Option<&str> {
        self.data["translated_name"].as_str()
    }
}

impl CheckUnknown for Tag {
    fn check_unknown(&self) -> Result<(), String> {
        check_json_keys!("name"+, "translated_name");
        Ok(())
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tag")
            .field("name", &self.name())
            .field("translated_name", &self.translated_name())
            .finish()
    }
}
