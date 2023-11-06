use crate::pixiv_app::PixivRestrictType;
use crate::push::every_push::EveryPushTextType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PixivMode {
    /// All artworks
    All,
    /// R18 artworks
    R18,
}

impl PixivMode {
    pub fn is_r18(&self) -> bool {
        matches!(self, Self::R18)
    }
}

fn default_restrict() -> PixivRestrictType {
    PixivRestrictType::All
}

fn default_mode() -> PixivMode {
    PixivMode::All
}

fn default_max_len_used() -> usize {
    50
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushTaskPixivAction {
    Follow {
        #[serde(default = "default_restrict")]
        /// Follower's type
        ///
        /// Only supported when using Pixiv APP API.
        restrict: PixivRestrictType,
        #[serde(default = "default_mode")]
        /// Illust's type
        ///
        /// Only supported when using Pixiv Web API.
        mode: PixivMode,
    },
    Bookmarks {
        #[serde(default = "default_restrict")]
        /// Bookmarks' type
        restrict: PixivRestrictType,
        /// User ID
        uid: u64,
    },
    Illusts {
        /// User ID
        uid: u64,
        #[serde(default = "default_max_len_used")]
        /// Maximum count of checked artworks.
        ///
        /// Only supported when using Pixiv Web API.
        max_len_used: usize,
    },
}

fn default_pixiv_max_len() -> usize {
    100
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
    #[serde(default = "default_pixiv_max_len")]
    /// Max length of cached pushed artworks list
    pub max_len: usize,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    ///
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
    #[serde(default = "default_true")]
    /// Whether to allow failed
    pub allow_failed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushDeerConfig {
    /// Push server
    pub push_server: String,
    /// Push key
    pub pushkey: String,
    /// Push type
    pub typ: EveryPushTextType,
    #[serde(default = "defualt_author_locations")]
    /// Author locations
    ///
    /// Not supported when type is `Image`.
    pub author_locations: Vec<AuthorLocation>,
    #[serde(default = "default_true")]
    /// Whether to filter author name
    pub filter_author: bool,
    #[serde(default = "default_true")]
    /// Whether to add artwork link
    pub add_link: bool,
    #[serde(default = "default_true")]
    /// Whether to add artwork link to title
    ///
    /// Supported when type is `Text`.
    pub add_link_to_title: bool,
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
    #[serde(default = "default_true")]
    /// Whether to add image link
    ///
    /// Supported when type is `Text`.
    pub add_image_link: bool,
    #[serde(default = "default_true")]
    /// Whether to allow failed
    pub allow_failed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PushConfig {
    EveryPush(EveryPushConfig),
    PushDeer(PushDeerConfig),
}

impl PushConfig {
    /// Whether to allow failed
    pub fn allow_failed(&self) -> bool {
        match self {
            Self::EveryPush(config) => config.allow_failed,
            Self::PushDeer(config) => config.allow_failed,
        }
    }
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

    pub fn is_need_update(&self) -> bool {
        let now = Utc::now();
        let last_updated = self.last_updated;
        let ttl = self.ttl;
        now.timestamp() - last_updated.timestamp() > ttl as i64
    }
}
