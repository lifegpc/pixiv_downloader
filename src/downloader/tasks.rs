use crate::ext::io::ClearFile;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::try_err::TryErr;
use crate::gettext;
use super::downloader::GetTargetFileName;
use super::error::DownloaderError;
use super::downloader::DownloaderInternal;
use futures_util::StreamExt;
use http_content_range::ContentRange;
use reqwest::Response;
use spin_on::spin_on;
use std::ops::Deref;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

/// Create a download tasks in simple thread mode.
pub async fn create_download_tasks_simple<T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName>(d: Arc<DownloaderInternal<T>>) -> Result<(), DownloaderError> {
    let mut start = if d.pd.is_downloading() {
        d.pd.get_downloaded_file_size()
    } else {
        0
    };
    let file_size = d.pd.get_file_size();
    let mut headers = d.headers.deref().clone();
    if start != 0 {
        match d.seek(SeekFrom::Start(start)) {
            Ok(data) => {
                if data != start {
                    start = 0;
                }
            }
            Err(_) => {
                start = 0;
            }
        }
        if start == 0 {
            d.seek(SeekFrom::Start(0))?;
            d.pd.clear()?;
        }
    }
    if start != 0 {
        headers.insert(String::from("Range"), format!("bytes={}-", start));
    }
    let mut result = d.client.aget(d.url.deref().clone(), headers).await.try_err(gettext("Failed to get url."))?;
    let mut status = result.status();
    if status == 416 {
        result = d.client.aget(d.url.deref().clone(), d.headers.deref().clone()).await.try_err(gettext("Failed to get url."))?;
        status = result.status();
    } else if status == 206 {
        let headers = result.headers();
        let need_reget = if headers.contains_key("Content-Range") {
            match ContentRange::parse_bytes(headers["Content-Range"].as_bytes()) {
                ContentRange::Bytes(b) => {
                    if file_size != 0 && b.complete_length != file_size {
                        true
                    } else if start != b.first_byte {
                        true
                    } else {
                        false
                    }
                }
                ContentRange::UnboundBytes(b) => {
                    if start != b.first_byte {
                        true
                    } else {
                        false
                    }
                }
                ContentRange::Unknown => {
                    true
                }
                _ => { false }
            }
        } else {
            true
        };
        if need_reget {
            d.pd.clear()?;
            d.clear_file()?;
            result = d.client.get(d.url.deref().clone(), d.headers.deref().clone()).try_err(gettext("Failed to get url."))?;
            status = result.status();
        }
    }
    if status.as_u16() >= 400 {
        return Err(DownloaderError::from(status));
    }
    if file_size == 0 && status != 206 {
        match result.content_length() {
            Some(len) => {
                d.pd.set_file_size(len)?;
                if d.enabled_progress_bar() {
                    d.set_progress_bar_length(len);
                }
            }
            None => {}
        }
    }
    if d.enabled_progress_bar() {
        d.set_progress_bar_message(gettext("Downloading \"<loc>\".").replace("<loc>", d.get_file_name().as_str()));
    }
    handle_download(d, result).await
}

/// Handle download process
pub async fn handle_download<T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName>(d: Arc<DownloaderInternal<T>>, re: Response) -> Result<(), DownloaderError> {
    let mut stream = re.bytes_stream();
    let is_multi = d.is_multi_threads();
    loop {
        let mut n = stream.next();
        let re = tokio::time::timeout(std::time::Duration::from_secs(10), &mut n).await;
        match re {
            Ok(s) => {
                match s {
                    Some(data) => {
                        match data {
                            Ok(data) => {
                                if !is_multi {
                                    let len = data.len() as u64;
                                    d.pd.inc(len)?;
                                    if d.enabled_progress_bar() {
                                        d.inc_progress_bar(len);
                                    }
                                }
                                d.write(&data)?;
                            }
                            Err(e) => {
                                if !is_multi {
                                    d.pd.clear()?;
                                    if d.enabled_progress_bar() {
                                        d.set_progress_bar_position(0);
                                        d.set_progress_bar_message(format!("{} {}", gettext("Error when downloading file:"), e));
                                    }
                                }
                                return Err(DownloaderError::from(e));
                            }
                        }
                    }
                    None => {
                        if !is_multi {
                            d.pd.complete()?;
                            if d.enabled_progress_bar() {
                                d.finish_progress_bar_with_message(format!("{} {}", gettext("Downloaded file:"), d.get_target_file_name().unwrap_or(String::from("(unknown)"))));
                            }
                        }
                        break;
                    }
                }
            }
            Err(e) => {
                if !is_multi {
                    d.pd.clear()?;
                    if d.enabled_progress_bar() {
                        d.set_progress_bar_position(0);
                        d.set_progress_bar_message(format!("{} {}", gettext("Error when downloading file:"), e));
                    }
                }
                return Err(DownloaderError::from(e));
            }
        }
    }
    Ok(())
}

/// Check tasks are completed or not. And create new tasks if needed.
pub async fn check_tasks<T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + 'static>(d: Arc<DownloaderInternal<T>>) -> Result<(), DownloaderError> {
    if !d.is_multi_threads() {
        let task = tokio::spawn(create_download_tasks_simple(Arc::clone(&d)));
        d.add_task(task);
    }
    loop {
        tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        let mut need_break = false;
        {
            let mut tasks = d.tasks.get_mut();
            tasks.retain_mut(|task| {
                if task.is_finished() {
                    let re = spin_on(task).unwrap();
                    match re {
                        Ok(_) => {
                            if !d.is_multi_threads() {
                                d.set_downloaded();
                                need_break = true;
                            }
                        }
                        Err(_) => {
                            let task = tokio::spawn(create_download_tasks_simple(Arc::clone(&d)));
                            d.add_task(task);
                        }
                    }
                    false
                } else {
                    true
                }
            });
        }
        if !d.is_multi_threads() && d.tasks.get_ref().len() == 0 {
            need_break = true;
        }
        if need_break {
            break;
        }
    }
    Ok(())
}
