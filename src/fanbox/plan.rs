use super::error::FanboxAPIError;
use crate::gettext;
use crate::parser::json::parse_u64;
use json::JsonValue;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

/// Fanbox plan
pub struct FanboxPlan {
    /// Raw data
    pub data: JsonValue,
}

impl FanboxPlan {
    pub fn id(&self) -> Option<u64> {
        parse_u64(&self.data["id"])
    }
}

impl Debug for FanboxPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FanboxPlan")
            .field("id", &self.id())
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
