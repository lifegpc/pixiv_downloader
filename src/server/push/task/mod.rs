pub mod pixiv_follow;
pub mod pixiv_illusts;
pub mod pixiv_send_message;

use super::super::preclude::*;
use crate::db::push_task::PushTaskPixivAction;
use crate::db::{PushTask, PushTaskConfig};
use crate::get_helper;
use crate::task_manager::{MaxCount, TaskManagerWithId};
use futures_util::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval_at, Duration, Instant};

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
    task: Arc<PushTask>,
    send_mode: Option<&TestSendMode>,
) -> Result<(), PixivDownloaderError> {
    match &task.config {
        PushTaskConfig::Pixiv(config) => match &config.act {
            PushTaskPixivAction::Follow { restrict, mode } => {
                pixiv_follow::run_push_task(ctx, task.clone(), config, restrict, mode, send_mode)
                    .await
            }
            PushTaskPixivAction::Illusts { uid, max_len_used } => {
                pixiv_illusts::run_push_task(
                    ctx,
                    task.clone(),
                    config,
                    uid.clone(),
                    max_len_used.clone(),
                    send_mode,
                )
                .await
            }
            _ => Ok(()),
        },
    }
}

pub async fn run_checking(ctx: Arc<ServerContext>) {
    let mut interval = interval_at(Instant::now(), Duration::from_secs(1));
    let manager = TaskManagerWithId::new(
        Arc::new(Mutex::new(0)),
        MaxCount::new(get_helper().push_task_max_count()),
    );
    loop {
        interval.tick().await;
        manager.check_task().await;
        let tasks = manager.take_finished_tasks();
        for (id, task) in tasks {
            let re = task.await;
            if let Ok(Err(e)) = re {
                log::warn!("Push task error (task id: {}): {}", id, e);
            } else if let Err(e) = re {
                log::error!("Join error: {}", e);
            } else if let Ok(Ok(())) = re {
                log::debug!("Push task finished: {}", id);
            }
        }
        let all_tasks = match ctx.db.get_all_push_tasks().await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Get all push tasks error: {}", e);
                break;
            }
        };
        for task in all_tasks {
            if task.is_need_update() && !manager.is_pending_or_running(&task.id) {
                let task = Arc::new(task);
                manager
                    .add_pending_task(task.id, run_push_task(ctx.clone(), task, None))
                    .await;
            }
        }
    }
}
