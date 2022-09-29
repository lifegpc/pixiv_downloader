use super::auth::*;
use super::context::ServerContext;
use crate::task_manager::{MaxCount, TaskManager};
use std::sync::Arc;
use tokio::time::{interval_at, Duration, Instant};

pub async fn start_timer(ctx: Arc<ServerContext>) {
    let mut interval = interval_at(Instant::now(), Duration::from_secs(60));
    let task_count = Arc::new(futures_util::lock::Mutex::new(0usize));
    let max_count = MaxCount::new(8);
    let tasks = TaskManager::new(task_count, max_count);
    loop {
        interval.tick().await;
        tasks.add_task(revoke_expired_tokens(ctx.clone())).await;
        tasks.join().await;
        for task in tasks.take_finished_tasks() {
            let re = task.await;
            if let Ok(Err(e)) = re {
                println!("Timer task error: {}", e);
            }
        }
    }
}
