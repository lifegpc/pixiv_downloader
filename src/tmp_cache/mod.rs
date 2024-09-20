use crate::concat_pixiv_downloader_error;
use crate::db::PixivDownloaderDb;
use crate::downloader::{DownloaderHelper, DownloaderResult};
use crate::error::PixivDownloaderError;
use crate::get_helper;
use crate::utils::get_file_name_from_url;
use crate::webclient::ToHeaders;
use chrono::{DateTime, Utc};
use futures_util::lock::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub struct TmpCacheEntry {
    pub url: String,
    pub path: String,
    pub last_used: DateTime<Utc>,
}

pub struct TmpCache {
    used: Mutex<HashMap<String, ()>>,
    db: Arc<Box<dyn PixivDownloaderDb + Send + Sync>>,
    in_cleaning: Mutex<()>,
}

impl TmpCache {
    pub fn new(db: Arc<Box<dyn PixivDownloaderDb + Send + Sync>>) -> Self {
        Self {
            used: Mutex::new(HashMap::new()),
            db,
            in_cleaning: Mutex::new(()),
        }
    }

    async fn _get_cache<H: ToHeaders>(
        &self,
        url: &str,
        headers: H,
    ) -> Result<PathBuf, PixivDownloaderError> {
        match self.db.get_tmp_cache(url).await? {
            Some(ent) => {
                if tokio::fs::try_exists(&ent.path).await.unwrap_or(false) {
                    return Ok(PathBuf::from(&ent.path));
                }
            }
            None => match self.db.delete_tmp_cache(url).await {
                _ => {}
            },
        }
        let mut tmp_dir = get_helper().temp_dir();
        let u = get_file_name_from_url(url).unwrap_or_else(|| {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            hasher.finish().to_string()
        });
        let dh = DownloaderHelper::builder(url)?
            .file_name(&u)
            .headers(headers)
            .build();
        let d = dh.download_local(Some(true), &tmp_dir)?;
        match d {
            DownloaderResult::Ok(d) => {
                d.disable_progress_bar();
                d.download();
                d.join().await?;
            }
            DownloaderResult::Canceled => {
                return Err(PixivDownloaderError::from("Download canceled."));
            }
        }
        tmp_dir.push(u);
        match self
            .db
            .put_tmp_cache(url, tmp_dir.to_string_lossy().trim())
            .await
        {
            Ok(()) => {}
            Err(e) => {
                log::warn!(target: "tmp_cache", "Failed to write cache {} to database: {}", url, e);
            }
        }
        Ok(tmp_dir)
    }

    pub async fn get_cache<H: ToHeaders>(
        &self,
        url: &str,
        headers: H,
    ) -> Result<PathBuf, PixivDownloaderError> {
        self.wait_for_url(url).await;
        let re = self._get_cache(url, headers).await;
        self.remove_for_url(url).await;
        re
    }

    pub async fn remove_cache_entry(&self, ent: TmpCacheEntry) -> Result<(), PixivDownloaderError> {
        let t = self.in_cleaning.try_lock();
        if t.is_none() {
            return Ok(());
        }
        self.wait_for_url(&ent.url).await;
        match tokio::fs::remove_file(&ent.path).await {
            Ok(_) => {}
            Err(e) => {
                log::warn!(target: "tmp_cache", "Failed to remove cache {}: {}", ent.path, e);
            }
        }
        match self.db.delete_tmp_cache(&ent.url).await {
            Ok(_) => {}
            Err(e) => {
                self.remove_for_url(&ent.url).await;
                Err(e)?;
            }
        }
        self.remove_for_url(&ent.url).await;
        Ok(())
    }

    pub async fn remove_expired_cache(&self) -> Result<(), PixivDownloaderError> {
        let entries = self.db.get_tmp_caches(3600).await?;
        let mut err = Ok(());
        for ent in entries {
            let e = self.remove_cache_entry(ent).await;
            concat_pixiv_downloader_error!(err, e);
        }
        err
    }

    async fn remove_for_url(&self, url: &str) {
        let mut m = self.used.lock().await;
        m.remove(url);
    }

    async fn wait_for_url(&self, url: &str) {
        loop {
            {
                let mut m = self.used.lock().await;
                if !m.contains_key(url) {
                    m.insert(url.to_owned(), ());
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::new(0, 100_000_000)).await;
        }
    }
}
