use crate::ext::replace::ReplaceWith;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::opthelper::get_helper;
use futures_util::lock::Mutex;
use indicatif::MultiProgress;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::task::JoinHandle;

lazy_static! {
    #[doc(hidden)]
    static ref TOTAL_DOWNLOAD_TASK_COUNT: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    #[doc(hidden)]
    static ref TOTAL_POST_TASK_COUNT: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    #[doc(hidden)]
    static ref PROGRESS_BAR: Arc<MultiProgress> = Arc::new(MultiProgress::new());
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

pub struct MaxDownloadPostTasks {
    _unused: [u8; 0],
}

impl MaxDownloadPostTasks {
    pub fn new() -> Self {
        MaxDownloadPostTasks { _unused: [] }
    }
}

impl GetMaxCount for MaxDownloadPostTasks {
    fn get_max_count(&self) -> usize {
        get_helper().max_download_post_tasks()
    }
}

#[derive(Clone, Debug)]
pub struct MaxCount {
    max_count: usize,
}

impl MaxCount {
    pub fn new(max_count: usize) -> Self {
        MaxCount { max_count }
    }
}

impl GetMaxCount for MaxCount {
    fn get_max_count(&self) -> usize {
        self.max_count
    }
}

/// Task manager
pub struct TaskManager<T> {
    /// Current running task
    tasks: RwLock<Vec<JoinHandle<T>>>,
    /// Finished task
    finished_tasks: RwLock<Vec<JoinHandle<T>>>,
    /// Total task count
    task_count: Arc<Mutex<usize>>,
    max_count: Box<dyn GetMaxCount + Send + Sync>,
}

impl<O> TaskManager<O> {
    /// Create a new instance
    pub fn new<T: GetMaxCount + Send + Sync + 'static>(
        task_count: Arc<Mutex<usize>>,
        max_count: T,
    ) -> Self {
        Self {
            tasks: RwLock::new(Vec::new()),
            finished_tasks: RwLock::new(Vec::new()),
            task_count,
            max_count: Box::new(max_count),
        }
    }

    /// Create a new instance with post max count
    pub fn new_post() -> Self {
        Self::new(get_total_post_task_count(), MaxDownloadPostTasks::new())
    }

    /// Add a new task.
    pub async fn add_task<F>(&self, future: F)
    where
        F: Future<Output = O> + Send + 'static,
        F::Output: Send + 'static,
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
                    self.tasks.get_mut().push(tokio::task::spawn(future));
                    count.replace_with(*count + 1);
                    break;
                }
            }
            tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        }
    }

    /// Try add a new task, if queue is full, run future on local thread
    pub async fn add_task_else_run_local<F>(&self, future: F) -> Option<F::Output>
    where
        F: Future<Output = O> + Send + 'static,
        F::Output: Send + 'static,
    {
        let total_count = self.max_count.get_max_count();
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
                self.tasks.get_mut().push(tokio::task::spawn(future));
                count.replace_with(*count + 1);
                return None;
            }
        }
        Some(future.await)
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
    pub fn take_finished_tasks(&self) -> Vec<JoinHandle<O>> {
        self.finished_tasks.replace_with2(Vec::new())
    }
}

pub fn get_progress_bar() -> Arc<MultiProgress> {
    Arc::clone(&PROGRESS_BAR)
}

pub fn get_total_download_task_count() -> Arc<Mutex<usize>> {
    Arc::clone(&TOTAL_DOWNLOAD_TASK_COUNT)
}

pub fn get_total_post_task_count() -> Arc<Mutex<usize>> {
    Arc::clone(&TOTAL_POST_TASK_COUNT)
}

impl<O> Default for TaskManager<O> {
    fn default() -> Self {
        Self::new(get_total_download_task_count(), MaxDownloadTasks::new())
    }
}

/// Task manager with ID
pub struct TaskManagerWithId<K, T> {
    /// Current running task
    tasks: RwLock<HashMap<K, JoinHandle<T>>>,
    /// Finished task
    finished_tasks: RwLock<HashMap<K, JoinHandle<T>>>,
    /// Pending task
    pedding_tasks: RwLock<Vec<(K, Pin<Box<dyn Future<Output = T> + Send + Sync + 'static>>)>>,
    /// Total task count
    task_count: Arc<Mutex<usize>>,
    max_count: Box<dyn GetMaxCount + Send + Sync>,
}

impl<K, O> TaskManagerWithId<K, O>
where
    K: Eq + std::hash::Hash,
    O: Send + 'static,
{
    /// Create a new instance
    pub fn new<T: GetMaxCount + Send + Sync + 'static>(
        task_count: Arc<Mutex<usize>>,
        max_count: T,
    ) -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            finished_tasks: RwLock::new(HashMap::new()),
            pedding_tasks: RwLock::new(Vec::new()),
            task_count,
            max_count: Box::new(max_count),
        }
    }

    /// Add pending task
    pub async fn add_pending_task<F>(&self, id: K, future: F)
    where
        F: Future<Output = O> + Send + Sync + 'static,
        F::Output: Send + 'static,
    {
        let total_count = self.max_count.get_max_count();
        {
            let mut count = self.task_count.lock().await;
            if *count < total_count {
                self.tasks.get_mut().insert(id, tokio::task::spawn(future));
                count.replace_with(*count + 1);
                return;
            }
        }
        self.pedding_tasks.get_mut().push((id, Box::pin(future)));
    }

    /// Add a new task.
    /// Wait until there is a free slot.
    /// * `before_pedding` - If true, add task before pending tasks, else add task after pending tasks.
    pub async fn add_task<F>(&self, id: K, future: F, before_pedding: bool)
    where
        F: Future<Output = O> + Send + 'static,
        F::Output: Send + 'static,
    {
        let total_count = self.max_count.get_max_count();
        loop {
            {
                let mut count = self.task_count.lock().await;
                let tasks = self.tasks.replace_with2(HashMap::new());
                let mut new_tasks = HashMap::new();
                let mut new_count = *count;
                for (k, v) in tasks {
                    if v.is_finished() {
                        self.finished_tasks.get_mut().insert(k, v);
                        new_count -= 1;
                    } else {
                        new_tasks.insert(k, v);
                    }
                }
                if !before_pedding {
                    while new_count < total_count {
                        if let Some((k, v)) = self.pedding_tasks.get_mut().pop() {
                            new_tasks.insert(k, tokio::task::spawn(v));
                            new_count += 1;
                        } else {
                            break;
                        }
                    }
                }
                self.tasks.replace_with2(new_tasks);
                count.replace_with(new_count);
                if *count < total_count {
                    self.tasks.get_mut().insert(id, tokio::task::spawn(future));
                    count.replace_with(*count + 1);
                    break;
                }
            }
            tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        }
    }

    /// Check running tasks and run pending tasks
    pub async fn check_task(&self) {
        let total_count = self.max_count.get_max_count();
        let mut count = self.task_count.lock().await;
        let tasks = self.tasks.replace_with2(HashMap::new());
        let mut new_tasks = HashMap::new();
        let mut new_count = *count;
        for (k, v) in tasks {
            if v.is_finished() {
                self.finished_tasks.get_mut().insert(k, v);
                new_count -= 1;
            } else {
                new_tasks.insert(k, v);
            }
        }
        while new_count < total_count {
            if let Some((k, v)) = self.pedding_tasks.get_mut().pop() {
                new_tasks.insert(k, tokio::task::spawn(v));
                new_count += 1;
            } else {
                break;
            }
        }
        self.tasks.replace_with2(new_tasks);
        count.replace_with(new_count);
    }

    pub fn is_pending(&self, id: &K) -> bool {
        self.pedding_tasks.get_ref().iter().any(|(k, _)| k == id)
    }

    pub fn is_pending_or_running(&self, id: &K) -> bool {
        self.is_running(id) || self.is_pending(id)
    }

    pub fn is_running(&self, id: &K) -> bool {
        self.tasks.get_ref().contains_key(id)
    }

    /// Wait all tasks finished.
    pub async fn join(&self) {
        let total_count = self.max_count.get_max_count();
        loop {
            {
                let mut count = self.task_count.lock().await;
                let tasks = self.tasks.replace_with2(HashMap::new());
                if tasks.len() == 0 && self.pedding_tasks.get_ref().len() == 0 {
                    break;
                }
                let mut new_tasks = HashMap::new();
                let mut new_count = *count;
                for (k, v) in tasks {
                    if v.is_finished() {
                        self.finished_tasks.get_mut().insert(k, v);
                        new_count -= 1;
                    } else {
                        new_tasks.insert(k, v);
                    }
                }
                while new_count < total_count {
                    if let Some((k, v)) = self.pedding_tasks.get_mut().pop() {
                        new_tasks.insert(k, tokio::task::spawn(v));
                        new_count += 1;
                    } else {
                        break;
                    }
                }
                self.tasks.replace_with2(new_tasks);
                count.replace_with(new_count);
            }
            tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        }
    }

    /// Take all finished tasks
    pub fn take_finished_tasks(&self) -> HashMap<K, JoinHandle<O>> {
        self.finished_tasks.replace_with2(HashMap::new())
    }
}
