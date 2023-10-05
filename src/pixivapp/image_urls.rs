use super::check::CheckUnknown;
use json::JsonValue;
use proc_macros::check_json_keys;

pub struct ImageUrls {
    data: JsonValue,
}

impl ImageUrls {
    pub fn new(data: JsonValue) -> Self {
        Self { data }
    }

    pub fn square_medium(&self) -> Option<&str> {
        self.data["square_medium"].as_str()
    }

    pub fn medium(&self) -> Option<&str> {
        self.data["medium"].as_str()
    }

    pub fn large(&self) -> Option<&str> {
        self.data["large"].as_str()
    }

    pub fn original(&self) -> Option<&str> {
        self.data["original"].as_str()
    }
}

impl CheckUnknown for ImageUrls {
    fn check_unknown(&self) -> Result<(), String> {
        check_json_keys!("square_medium"+, "medium"+, "large"+, "original"+);
        Ok(())
    }
}

impl std::fmt::Debug for ImageUrls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageUrls")
            .field("square_medium", &self.square_medium())
            .field("medium", &self.medium())
            .field("large", &self.large())
            .field("original", &self.original())
            .finish()
    }
}
