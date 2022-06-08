use crate::ext::try_err::TryErr;
use crate::gettext;
use super::error::DownloaderError;
use super::downloader::DownloaderInternal;
use http_content_range::ContentRange;
use std::ops::Deref;
use std::io::Seek;
use std::io::Write;
use std::sync::Arc;

/// Create a download tasks in simple thread mode.
pub async fn create_download_tasks_simple<T: Seek + Write + Send + Sync>(d: Arc<DownloaderInternal<T>>) -> Result<(), DownloaderError> {
    let start = if d.pd.is_downloading() {
        d.pd.get_downloaded_file_size()
    } else {
        0
    };
    let file_size = d.pd.get_file_size();
    let mut headers = d.headers.deref().clone();
    if start != 0 {
        headers.insert(String::from("Range"), format!("bytes={}-", start));
    }
    let mut result = d.client.get(d.url.deref().clone(), headers).try_err(gettext("Failed to get url."))?;
    let mut status = result.status();
    if status == 416 {
        result = d.client.get(d.url.deref().clone(), d.headers.deref().clone()).try_err(gettext("Failed to get url."))?;
        status = result.status();
    } else if status == 206 {
        let range = ContentRange::parse_bytes(result.headers()["Content-Range"].as_bytes());
        let need_reget = match range {
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
        };
        if need_reget {
            d.pd.clear()?;
            result = d.client.get(d.url.deref().clone(), d.headers.deref().clone()).try_err(gettext("Failed to get url."))?;
            status = result.status();
        }
    }
    if status.as_u16() >= 400 {
        return Err(DownloaderError::from(status));
    }
    Ok(())
}
