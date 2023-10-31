use crate::pixiv_app::PixivRestrictType;
use crate::push::every_push::EveryPushTextType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushTaskPixivAction {
    Follow {
        restrict: PixivRestrictType,
    },
    Bookmarks {
        restrict: PixivRestrictType,
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
    /// Whether to use description from Web API when description from APP API is empty.
    pub use_web_description: Option<bool>,
    /// Whether to use Pixiv APP API first.
    pub use_app_api: Option<bool>,
    /// Use data from webpage first.
    pub use_webpage: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushTaskConfig {
    Pixiv(PushTaskPixivConfig),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
/// Author location
pub enum AuthorLocation {
    /// add author name to title
    Title,
    /// add author name to the top of description
    Top,
    /// add author name to the bottom of description
    Bottom,
}

fn defualt_author_locations() -> Vec<AuthorLocation> {
    vec![AuthorLocation::Top]
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EveryPushConfig {
    /// Push server
    pub push_server: String,
    /// Push token
    pub push_token: String,
    /// Push type
    pub typ: EveryPushTextType,
    /// Push topic ID
    pub topic_id: Option<String>,
    #[serde(default = "defualt_author_locations")]
    /// Author locations
    /// If type is `Image`, this field only support [AuthorLocation::Title].
    pub author_locations: Vec<AuthorLocation>,
    #[serde(default = "default_true")]
    /// Whether to filter author name
    pub filter_author: bool,
    #[serde(default = "default_true")]
    /// Whether to add artwork link
    pub add_link: bool,
    #[serde(default = "default_true")]
    /// Whether to add image link to image
    pub add_link_to_image: bool,
    #[serde(default = "default_true")]
    /// Whether to add tags
    pub add_tags: bool,
    #[serde(default = "default_true")]
    /// Whether to add AI tag
    pub add_ai_tag: bool,
    #[serde(default = "default_true")]
    /// Whether to add translated tag
    pub add_translated_tag: bool,
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

impl PushTask {
    pub fn new(config: PushTaskConfig, push_configs: Vec<PushConfig>) -> Self {
        Self {
            id: 0,
            config,
            push_configs,
            last_updated: DateTime::UNIX_EPOCH,
            ttl: 0,
        }
    }
}
