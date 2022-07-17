use crate::ext::any::AsAny;
use crate::ext::replace::ReplaceWith;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::opthelper::get_helper;
use futures_util::lock::Mutex;
use indicatif::MultiProgress;
use std::any::Any;
use std::future::Future;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::task::JoinHandle;

lazy_static! {
    #[doc(hidden)]
    static ref TOTAL_DOWNLOAD_TASK_COUNT: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    #[doc(hidden)]
    static ref PROGRESS_BAR: Arc<MultiProgress> = Arc::new(MultiProgress::new());
}

pub trait IsFinished {
    fn is_finished(&self) -> bool;
}

impl<T> IsFinished for JoinHandle<T> {
    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}

pub trait IsFinishedAny: IsFinished + Any {}

impl<T> IsFinishedAny for T where T: IsFinished + Any {}

impl AsAny<dyn Any + Send + Sync + 'static> for Box<dyn IsFinishedAny + Send + Sync> {
    fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync + 'static) {
        self
    }
}

pub trait GetMaxCount {
    fn get_max_count(&self) -> usize;
}

pub struct MaxDownloadTasks {
    _unused: [u8; 0],
}

impl MaxDownloadTasks {
    pub fn new() -> Self {
        MaxDownloadTasks { _unused: [] }
    }
}

impl GetMaxCount for MaxDownloadTasks {
    fn get_max_count(&self) -> usize {
        get_helper().max_download_tasks()
    }
}

/// Task manager
pub struct TaskManager {
    /// Current running task
    tasks: RwLock<Vec<Box<dyn IsFinishedAny + Send + Sync>>>,
    /// Finished task
    finished_tasks: RwLock<Vec<Box<dyn IsFinishedAny + Send + Sync>>>,
    /// Total task count
    task_count: Arc<Mutex<usize>>,
    max_count: Box<dyn GetMaxCount>,
}

impl TaskManager {
    /// Create a new instance
    pub fn new<T: GetMaxCount + 'static>(task_count: Arc<Mutex<usize>>, max_count: T) -> Self {
        Self {
            tasks: RwLock::new(Vec::new()),
            finished_tasks: RwLock::new(Vec::new()),
            task_count,
            max_count: Box::new(max_count),
        }
    }

    /// Add a new task.
    pub async fn add_task<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
        JoinHandle<F::Output>: IsFinishedAny + Send + Sync + 'static,
    {
        let total_count = self.max_count.get_max_count();
        loop {
            {
                let mut count = self.task_count.lock().await;
                let tasks = self.tasks.replace_with2(Vec::new());
                let mut new_tasks = Vec::new();
                let mut new_count = *count;
                for i in tasks {
                    if i.is_finished() {
                        self.finished_tasks.get_mut().push(i);
                        new_count -= 1;
                    } else {
                        new_tasks.push(i);
                    }
                }
                self.tasks.replace_with2(new_tasks);
                count.replace_with(new_count);
                if *count < total_count {
                    self.tasks
                        .get_mut()
                        .push(Box::new(tokio::task::spawn(future)));
                    count.replace_with(*count + 1);
                    break;
                }
            }
            tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        }
    }

    /// Wait all tasks finished.
    pub async fn join(&self) {
        loop {
            {
                let mut count = self.task_count.lock().await;
                let tasks = self.tasks.replace_with2(Vec::new());
                if tasks.len() == 0 {
                    break;
                }
                let mut new_tasks = Vec::new();
                let mut new_count = *count;
                for i in tasks {
                    if i.is_finished() {
                        self.finished_tasks.get_mut().push(i);
                        new_count -= 1;
                    } else {
                        new_tasks.push(i);
                    }
                }
                self.tasks.replace_with2(new_tasks);
                count.replace_with(new_count);
            }
            tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        }
    }

    /// Take all finished tasks
    pub fn take_finished_tasks(&self) -> Vec<Box<dyn IsFinishedAny + Send + Sync>> {
        self.finished_tasks.replace_with2(Vec::new())
    }
}

pub fn get_progress_bar() -> Arc<MultiProgress> {
    Arc::clone(&PROGRESS_BAR)
}

pub fn get_total_download_task_count() -> Arc<Mutex<usize>> {
    Arc::clone(&TOTAL_DOWNLOAD_TASK_COUNT)
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new(get_total_download_task_count(), MaxDownloadTasks::new())
    }
}
