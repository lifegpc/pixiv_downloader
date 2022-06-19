use super::downloader::DownloaderInternal;
use super::downloader::GetTargetFileName;
use super::error::DownloaderError;
use crate::ext::io::ClearFile;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::try_err::TryErr;
use crate::gettext;
use futures_util::StreamExt;
use http_content_range::ContentRange;
use itertools::partition;
use reqwest::Response;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

/// Return [Ok(())] if the [super::Downloader] is dropped.
macro_rules! check_dropped {
    ($d:expr) => {
        if $d.is_dropped() {
            #[cfg(test)]
            {
                println!("The downloader status: {}", $d.get_status());
                if $d.is_downloading() {
                    println!(
                        "The downloader size: {} / {}",
                        $d.pd.get_downloaded_file_size(),
                        $d.pd.get_file_size()
                    );
                }
                println!("The downloader is already dropped. Exit download.");
            }
            return Ok(());
        }
    };
}

/// Create a download tasks in simple thread mode.
pub async fn create_download_tasks_simple<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName,
>(
    d: Arc<DownloaderInternal<T>>,
) -> Result<(), DownloaderError> {
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
    let mut result = d
        .client
        .get(d.url.deref().clone(), headers)
        .await
        .try_err(gettext("Failed to get url."))?;
    let mut status = result.status();
    if status == 416 {
        result = d
            .client
            .get(d.url.deref().clone(), d.headers.deref().clone())
            .await
            .try_err(gettext("Failed to get url."))?;
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
                ContentRange::Unknown => true,
                _ => false,
            }
        } else {
            true
        };
        if need_reget {
            d.pd.clear()?;
            d.clear_file()?;
            result = d
                .client
                .get(d.url.deref().clone(), d.headers.deref().clone())
                .await
                .try_err(gettext("Failed to get url."))?;
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
        d.set_progress_bar_message(
            gettext("Downloading \"<loc>\".").replace("<loc>", d.get_file_name().as_str()),
        );
    }
    handle_download(d, result).await
}

/// Do first job when download in multiple mode.
pub async fn create_download_tasks_multi_first<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName,
>(
    d: Arc<DownloaderInternal<T>>,
) -> Result<(), DownloaderError> {
    let result = d
        .client
        .get(d.url.deref().clone(), d.headers.as_ref().clone())
        .await
        .try_err(gettext("Failed to get url."))?;
    let status = result.status();
    if status.as_u16() >= 400 {
        return Err(DownloaderError::from(status));
    }
    // # TODO
    Ok(())
}

/// Handle download process
pub async fn handle_download<T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName>(
    d: Arc<DownloaderInternal<T>>,
    re: Response,
) -> Result<(), DownloaderError> {
    let mut stream = re.bytes_stream();
    let is_multi = d.is_multi_threads();
    loop {
        let mut n = stream.next();
        check_dropped!(d);
        let re = tokio::time::timeout(std::time::Duration::from_secs(10), &mut n).await;
        match re {
            Ok(s) => match s {
                Some(data) => match data {
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
                                d.set_progress_bar_message(format!(
                                    "{} {}",
                                    gettext("Error when downloading file:"),
                                    e
                                ));
                            }
                        }
                        return Err(DownloaderError::from(e));
                    }
                },
                None => {
                    if !is_multi {
                        d.pd.complete()?;
                        if d.enabled_progress_bar() {
                            d.finish_progress_bar_with_message(format!(
                                "{} {}",
                                gettext("Downloaded file:"),
                                d.get_target_file_name()
                                    .unwrap_or(String::from("(unknown)"))
                            ));
                        }
                    }
                    break;
                }
            },
            Err(e) => {
                if !is_multi {
                    d.pd.clear()?;
                    if d.enabled_progress_bar() {
                        d.set_progress_bar_position(0);
                        d.set_progress_bar_message(format!(
                            "{} {}",
                            gettext("Error when downloading file:"),
                            e
                        ));
                    }
                }
                return Err(DownloaderError::from(e));
            }
        }
    }
    Ok(())
}

/// Check tasks are completed or not. And create new tasks if needed.
pub async fn check_tasks<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + 'static,
>(
    d: Arc<DownloaderInternal<T>>,
) -> Result<(), DownloaderError> {
    if !d.is_multi_threads() {
        let task = tokio::spawn(create_download_tasks_simple(Arc::clone(&d)));
        d.add_task(task);
    } else {
        if d.pd.is_started() {
            let task = tokio::spawn(create_download_tasks_multi_first(Arc::clone(&d)));
            d.add_task(task);
        }
    }
    loop {
        check_dropped!(d);
        tokio::time::sleep(Duration::new(0, 10_000_000)).await;
        let mut need_break = false;
        let mut dur = None;
        {
            let mut tasks = d.tasks.replace_with2(Vec::new());
            let mut index = partition(&mut tasks, |s| s.is_finished());
            while index > 0 {
                let task = tasks.remove(0);
                let re = task.await.unwrap();
                match re {
                    Ok(_) => {
                        if !d.is_multi_threads() {
                            d.set_downloaded();
                            need_break = true;
                        }
                    }
                    Err(e) => {
                        println!("{}", e);
                        if !d.is_multi_threads() {
                            match d.get_retry_duration() {
                                Some(d) => dur = Some(d),
                                None => {
                                    d.set_panic(e);
                                    need_break = true;
                                }
                            }
                        }
                    }
                }
                index -= 1;
            }
            d.tasks.replace_with2(tasks);
        }
        if !d.is_multi_threads() && dur.is_some() {
            let dur = dur.unwrap();
            if !dur.is_zero() {
                tokio::time::sleep(dur).await;
            }
            let task = tokio::spawn(create_download_tasks_simple(Arc::clone(&d)));
            d.add_task(task);
        }
        if need_break {
            break;
        }
    }
    if d.is_panic() {
        let tasks = d.tasks.get_ref();
        for i in tasks.iter() {
            i.abort();
        }
    }
    Ok(())
}
