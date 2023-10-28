pub mod pixiv_follow;
pub mod pixiv_send_message;

use super::super::preclude::*;
use crate::db::push_task::PushTaskPixivAction;
use crate::db::{PushTask, PushTaskConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestSendMode {
    /// Send first message
    First,
    /// Send last message
    Last,
    /// Send random message
    Random,
    /// Send all messages
    All,
    #[serde(untagged)]
    /// Send specified message
    Fixed(u64),
}

impl TestSendMode {
    pub fn is_all(&self) -> bool {
        matches!(self, TestSendMode::All)
    }

    pub fn to_index(&self, len: usize) -> Option<usize> {
        match self {
            TestSendMode::First => Some(0),
            TestSendMode::Last => Some(len - 1),
            TestSendMode::Random => {
                if len == 0 {
                    None
                } else {
                    Some(rand::random::<usize>() % len)
                }
            }
            TestSendMode::All => None,
            TestSendMode::Fixed(index) => {
                if *index >= len as u64 {
                    None
                } else {
                    Some(*index as usize)
                }
            }
        }
    }
}

pub async fn run_push_task(
    ctx: Arc<ServerContext>,
    task: &PushTask,
    send_mode: Option<&TestSendMode>,
) -> Result<(), PixivDownloaderError> {
    match &task.config {
        PushTaskConfig::Pixiv(config) => match &config.act {
            PushTaskPixivAction::Follow { restrict } => {
                pixiv_follow::run_push_task(ctx, task, config, restrict, send_mode).await
            }
            _ => Ok(()),
        },
    }
}
