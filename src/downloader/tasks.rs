use super::downloader::DownloaderInternal;
use std::io::Seek;
use std::io::Write;
use std::sync::Arc;

/// Create a download tasks in simple thread mode.
pub async fn create_download_tasks_simple<T: Seek + Write + Send + Sync>(d: Arc<DownloaderInternal<T>>) -> bool {
    false
}
