use super::check::CheckUnknown;
use super::error::FanboxAPIError;
use crate::gettext;
use crate::parser::json::parse_u64;
use json::JsonValue;
use proc_macros::check_json_keys;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

/// Fanbox plan
pub struct FanboxPlan {
    /// Raw data
    pub data: JsonValue,
}

impl FanboxPlan {
    #[inline]
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }

    #[inline]
    pub fn cover_image_url(&self) -> Option<&str> {
        self.data["coverImageUrl"].as_str()
    }

    #[inline]
    pub fn creator_id(&self) -> Option<&str> {
        self.data["creatorId"].as_str()
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.data["description"].as_str()
    }

    #[inline]
    pub fn fee(&self) -> Option<u64> {
        self.data["fee"].as_u64()
    }

    #[inline]
    pub fn has_adult_content(&self) -> Option<bool> {
        self.data["hasAdultContent"].as_bool()
    }

    #[inline]
    pub fn payment_method(&self) -> Option<&str> {
        self.data["paymentMethod"].as_str()
    }

    #[inline]
    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    #[inline]
    pub fn user_icon_url(&self) -> Option<&str> {
        self.data["user"]["iconUrl"].as_str()
    }

    #[inline]
    pub fn user_id(&self) -> Option<u64> {
        parse_u64(&self.data["user"]["userId"])
    }

    #[inline]
    pub fn user_name(&self) -> Option<&str> {
        self.data["user"]["name"].as_str()
    }
}

impl CheckUnknown for FanboxPlan {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        check_json_keys!(
            "id"+,
            "coverImageUrl",
            "creatorId"+,
            "description"+,
            "fee"+,
            "hasAdultContent"+,
            "paymentMethod",
            "title"+,
            "user": [
                "iconUrl",
                "userId"+user_id,
                "name"+,
            ],
        );
        Ok(())
    }
}

impl Debug for FanboxPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPlan")
            .field("id", &self.id())
            .field("cover_image_url", &self.cover_image_url())
            .field("creator_id", &self.creator_id())
            .field("description", &self.description())
            .field("fee", &self.fee())
            .field("has_adult_content", &self.has_adult_content())
            .field("payment_method", &self.payment_method())
            .field("title", &self.title())
            .field("user_icon_url", &self.user_icon_url())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .finish_non_exhaustive()
    }
}

impl From<JsonValue> for FanboxPlan {
    fn from(data: JsonValue) -> Self {
        Self { data }
    }
}

impl From<&JsonValue> for FanboxPlan {
    fn from(data: &JsonValue) -> Self {
        Self { data: data.clone() }
    }
}

/// A list of fanbox plans
pub struct FanboxPlanList {
    /// List
    list: Vec<FanboxPlan>,
}

impl FanboxPlanList {
    /// Create a new instance.
    /// * `data` - The data returned from the fanbox API.
    pub fn new(data: &JsonValue) -> Result<Self, FanboxAPIError> {
        if data.is_array() {
            let mut list = Vec::new();
            for i in data.members() {
                if !i.is_object() {
                    return Err(FanboxAPIError::from(gettext(
                        "Failed to parse fanbox plan list.",
                    )));
                }
                list.push(FanboxPlan::from(i));
            }
            Ok(Self { list })
        } else {
            Err(FanboxAPIError::from(gettext(
                "Failed to parse fanbox plan list.",
            )))
        }
    }
}

impl CheckUnknown for FanboxPlanList {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        for i in self.list.iter() {
            i.check_unknown()?;
        }
        Ok(())
    }
}

impl Debug for FanboxPlanList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.list.iter()).finish()
    }
}

impl Deref for FanboxPlanList {
    type Target = Vec<FanboxPlan>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for FanboxPlanList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
