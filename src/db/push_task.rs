use crate::push::every_push::EveryPushTextType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PushTaskPixivRestrictType {
    Public,
    Private,
    All,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushTaskPixivAction {
    Follow {
        restrict: PushTaskPixivRestrictType,
    },
    Bookmarks {
        restrict: PushTaskPixivRestrictType,
        uid: u64,
    },
    Illusts {
        uid: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushTaskPixivConfig {
    pub act: PushTaskPixivAction,
    /// Tag translation language
    pub lang: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushTaskConfig {
    Pixiv(PushTaskPixivConfig),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveryPushConfig {
    /// Push server
    pub push_server: String,
    /// Push token
    pub push_token: String,
    #[serde(rename = "type")]
    /// Push type
    pub typ: EveryPushTextType,
    /// Push topic ID
    pub topic_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushConfig {
    EveryPush(EveryPushConfig),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushTask {
    /// The task ID
    pub id: u64,
    /// Configurations of the task
    pub config: PushTaskConfig,
    /// Push configurations
    pub push_configs: Vec<PushConfig>,
    #[serde(with = "chrono::serde::ts_seconds")]
    /// Last updated time
    pub last_updated: DateTime<Utc>,
    /// Update interval
    pub ttl: u64,
}
