use super::downloader::DownloaderInternal;
use super::downloader::GetTargetFileName;
use super::downloader::SetLen;
use super::error::DownloaderError;
use super::pd_file::PdFilePartStatus;
use crate::concat_error;
use crate::ext::atomic::AtomicQuick;
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
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen,
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
                d.set_len(len)?;
            }
            None => {}
        }
    }
    if d.enabled_progress_bar() {
        d.set_progress_bar_message(
            gettext("Downloading \"<loc>\".").replace("<loc>", d.get_file_name().as_str()),
        );
    }
    handle_download(d, result, None, None).await
}

/// Do first job when download in multiple mode.
pub async fn create_download_tasks_multi_first<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen,
>(
    d: Arc<DownloaderInternal<T>>,
) -> Result<(), DownloaderError> {
    #[cfg(test)]
    {
        println!("Created first download task in multiple thread mode.");
    }
    let result = d
        .client
        .get(d.url.deref().clone(), d.headers.as_ref().clone())
        .await
        .try_err(gettext("Failed to get url."))?;
    let status = result.status();
    #[cfg(test)]
    {
        println!("HTTP status: {}", status);
    }
    if status.as_u16() >= 400 {
        return Err(DownloaderError::from(status));
    }
    match result.content_length() {
        Some(len) => {
            match d.pd.set_file_size(len) {
                Ok(_) => {
                    #[cfg(test)]
                    {
                        println!("Set the file size to {}", len);
                        println!("Is downloading: {}", d.pd.is_downloading());
                    }
                }
                Err(e) => {
                    log::error!("{}", e)
                }
            }
            if d.enabled_progress_bar() {
                d.set_progress_bar_length(len);
            }
            d.set_len(len)?;
        }
        None => {
            d.fallback_to_simp();
            return Err(DownloaderError::from(gettext(
                "Warning: no content-length, fallback to single thread download.",
            )));
        }
    }
    match d.pd.initialize_part_datas() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{}", e);
        }
    }
    if d.enabled_progress_bar() {
        d.set_progress_bar_message(
            gettext("Downloading \"<loc>\".").replace("<loc>", d.get_file_name().as_str()),
        );
    }
    Ok(())
}

/// Create a new download task in multiple thread mode.
pub async fn create_download_tasks_multi<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen,
>(
    d: Arc<DownloaderInternal<T>>,
    pd: Arc<PdFilePartStatus>,
    index: usize,
) -> Result<(), DownloaderError> {
    let mut re = create_download_tasks_multi_internal(d, Arc::clone(&pd), index).await;
    if re.is_err() {
        concat_error!(re, pd.set_waited(), DownloaderError);
        concat_error!(re, pd.set_downloaded_size(0), DownloaderError);
    }
    re
}

/// Create a new download task in multiple thread mode.
pub async fn create_download_tasks_multi_internal<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen,
>(
    d: Arc<DownloaderInternal<T>>,
    pd: Arc<PdFilePartStatus>,
    index: usize,
) -> Result<(), DownloaderError> {
    let part_size = d.get_part_size() as u64;
    let file_size = d.pd.get_file_size();
    let start = part_size * (index as u64);
    let end = std::cmp::min(start + part_size - 1, file_size);
    let mut headers = d.headers.deref().clone();
    headers.insert(String::from("Range"), format!("bytes={}-{}", start, end));
    let result = d
        .client
        .get(d.url.deref().clone(), headers)
        .await
        .try_err(gettext("Failed to get url."))?;
    let status = result.status();
    #[cfg(test)]
    {
        println!("Index {}: HTTP Status {}", index, status);
    }
    if status == 200 || status == 416 {
        // FIX ME: FALLBACK will cause download not completed.
        // d.fallback_to_simp();
        // d.tasks.replace_with2(Vec::new());
        return Err(DownloaderError::from(gettext(
            "Warning: The server seems does not support range.",
        )));
    }
    if status.as_u16() != 206 {
        return Err(DownloaderError::from(status));
    }
    handle_download(d, result, Some(pd), Some(index)).await
}

/// Handle download process
pub async fn handle_download<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen,
>(
    d: Arc<DownloaderInternal<T>>,
    re: Response,
    pd: Option<Arc<PdFilePartStatus>>,
    index: Option<usize>,
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
                            d.write(&data)?;
                        } else {
                            if !d.is_multi_threads() {
                                return Ok(());
                            }
                            let len = data.len() as u32;
                            d.write_part(&data, pd.as_ref().unwrap(), index.unwrap())?;
                            pd.as_ref().unwrap().inc(len)?;
                            d.pd.inc(len as u64)?;
                            d.pd.update_part_data(index.unwrap())?;
                            #[cfg(test)]
                            {
                                println!("Index {}: Writed data {} bytes", index.unwrap(), len);
                            }
                        }
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
                        } else {
                            if !d.is_multi_threads() {
                                return Ok(());
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
                    } else {
                        if !d.is_multi_threads() {
                            return Ok(());
                        }
                        #[cfg(test)]
                        {
                            println!("Index {} set to downloaded.", index.unwrap());
                        }
                        pd.as_ref().unwrap().set_downloaded()?;
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
                } else {
                    if !d.is_multi_threads() {
                        return Ok(());
                    }
                }
                return Err(DownloaderError::from(e));
            }
        }
    }
    Ok(())
}

pub async fn add_new_multi_tasks<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen + 'static,
>(
    d: &Arc<DownloaderInternal<T>>,
) -> Result<(), DownloaderError> {
    let mut needed_size = (d.max_threads.qload() as usize) - d.tasks.get_ref().len();
    while needed_size > 0 {
        check_dropped!(d);
        let mut data = None;
        let index = d.pd.get_next_waited_part_data(&mut data);
        match index {
            Some(index) => {
                data.as_ref().unwrap().set_downloading().unwrap();
                let task = tokio::spawn(create_download_tasks_multi(
                    Arc::clone(d),
                    data.unwrap(),
                    index,
                ));
                d.add_task(task);
            }
            None => {
                return Ok(());
            }
        }
        needed_size -= 1;
    }
    Ok(())
}

/// Check tasks are completed or not. And create new tasks if needed.
pub async fn check_tasks<
    T: Seek + Write + Send + Sync + ClearFile + GetTargetFileName + SetLen + 'static,
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
        } else {
            add_new_multi_tasks(&d).await?;
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
                        log::error!("{}", e);
                        if !d.is_multi_threads() {
                            match d.get_retry_duration() {
                                Some(d) => dur = Some(d),
                                None => {
                                    d.set_panic(e);
                                    need_break = true;
                                }
                            }
                        } else {
                            if d.pd.is_started() {
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
        } else if d.is_multi_threads() {
            if d.pd.is_started() {
                if d.tasks.get_ref().len() == 0 {
                    match dur {
                        Some(dur) => {
                            if !dur.is_zero() {
                                tokio::time::sleep(dur).await;
                            }
                            let task =
                                tokio::spawn(create_download_tasks_multi_first(Arc::clone(&d)));
                            d.add_task(task);
                        }
                        None => {}
                    }
                }
            } else {
                if d.enabled_progress_bar() {
                    d.set_progress_bar_position(d.pd.get_downloaded_file_size());
                }
                if d.tasks.get_ref().len() < (d.max_threads.qload() as usize) {
                    add_new_multi_tasks(&d).await?;
                }
                if d.pd.is_all_part_downloaded() {
                    match d.pd.complete() {
                        Ok(_) => {
                            need_break = true;
                            d.set_downloaded();
                            if d.enabled_progress_bar() {
                                d.finish_progress_bar_with_message(format!(
                                    "{} {}",
                                    gettext("Downloaded file:"),
                                    d.get_target_file_name()
                                        .unwrap_or(String::from("(unknown)"))
                                ));
                            }
                        }
                        Err(e) => {
                            log::error!("{}", e);
                        }
                    }
                }
            }
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
