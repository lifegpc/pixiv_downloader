use crate::ext::io::ClearFile;
use crate::ext::try_err::TryErr;
use crate::gettext;
use super::error::DownloaderError;
use super::downloader::DownloaderInternal;
use futures_util::StreamExt;
use http_content_range::ContentRange;
use reqwest::Response;
use std::ops::Deref;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::sync::Arc;

/// Create a download tasks in simple thread mode.
pub async fn create_download_tasks_simple<T: Seek + Write + Send + Sync + ClearFile>(d: Arc<DownloaderInternal<T>>) -> Result<(), DownloaderError> {
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
    let mut result = d.client.get(d.url.deref().clone(), headers).try_err(gettext("Failed to get url."))?;
    let mut status = result.status();
    if status == 416 {
        result = d.client.get(d.url.deref().clone(), d.headers.deref().clone()).try_err(gettext("Failed to get url."))?;
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
            }
            None => {}
        }
    }
    handle_download(d, result).await
}

/// Handle download process
pub async fn handle_download<T: Seek + Write + Send + Sync + ClearFile>(d: Arc<DownloaderInternal<T>>, re: Response) -> Result<(), DownloaderError> {
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
                                d.pd.inc(data.len() as u64)?;
                                d.write(&data)?;
                            }
                            Err(e) => {
                                if !is_multi {
                                    d.pd.clear()?;
                                }
                                return Err(DownloaderError::from(e));
                            }
                        }
                    }
                    None => {
                        if !is_multi {
                            d.pd.complete()?;
                        }
                        break;
                    }
                }
            }
            Err(e) => {
                if !is_multi {
                    d.pd.clear()?;
                }
                return Err(DownloaderError::from(e));
            }
        }
    }
    Ok(())
}
