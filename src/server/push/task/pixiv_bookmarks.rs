use super::super::super::preclude::*;
use super::pixiv_send_message::send_message;
use super::TestSendMode;
use crate::db::push_task::{PushTask, PushTaskPixivConfig};
use crate::ext::atomic::AtomicQuick;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::pixiv_app::{PixivRestrictLessType, PixivRestrictType};
use crate::pixivapp::illust::PixivAppIllust;
use crate::task_manager::{MaxCount, TaskManagerWithId};
use crate::utils::parse_pixiv_id;
use crate::{concat_pixiv_downloader_error, get_helper};
use futures_util::lock::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

struct PixivBookmarksData {
    web_data: RwLock<HashMap<u64, JsonValue>>,
}

impl PixivBookmarksData {
    pub fn new() -> Self {
        Self {
            web_data: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_web_data(&self, id: u64) -> Option<JsonValue> {
        self.web_data.get_ref().get(&id).cloned()
    }

    pub fn set_web_data(&self, id: u64, data: JsonValue) {
        self.web_data.get_mut().insert(id, data);
    }
}

struct RunContext<'a> {
    ctx: Arc<ServerContext>,
    task: Arc<PushTask>,
    config: &'a PushTaskPixivConfig,
    uid: u64,
    restrict: &'a PixivRestrictType,
    tag: Option<&'a str>,
    send_mode: Option<&'a TestSendMode>,
    data: Arc<PixivBookmarksData>,
    use_app_api: bool,
    use_web_description: bool,
    use_webpage: bool,
    pushed: RwLock<Vec<u64>>,
    first_run: AtomicBool,
    push_manager: TaskManagerWithId<usize, Result<(), PixivDownloaderError>>,
}

impl<'a> RunContext<'a> {
    pub fn new(
        ctx: Arc<ServerContext>,
        task: Arc<PushTask>,
        config: &'a PushTaskPixivConfig,
        uid: u64,
        restrict: &'a PixivRestrictType,
        tag: Option<&'a str>,
        send_mode: Option<&'a TestSendMode>,
    ) -> Self {
        let helper = get_helper();
        Self {
            ctx,
            task,
            config,
            uid,
            restrict,
            tag,
            send_mode,
            data: Arc::new(PixivBookmarksData::new()),
            use_app_api: config.use_app_api.unwrap_or(helper.use_app_api()),
            use_web_description: config
                .use_web_description
                .unwrap_or(helper.use_web_description()),
            use_webpage: config.use_webpage.unwrap_or(helper.use_webpage()),
            pushed: RwLock::new(Vec::new()),
            first_run: AtomicBool::new(false),
            push_manager: TaskManagerWithId::new(
                Arc::new(Mutex::new(0)),
                MaxCount::new(helper.push_task_max_push_count()),
            ),
        }
    }

    pub async fn handle_finished_tasks(&self) -> Result<(), PixivDownloaderError> {
        let tasks = self.push_manager.take_finished_tasks();
        let mut error = Ok(());
        for (i, task) in tasks {
            let cfg = self
                .task
                .push_configs
                .get(i)
                .ok_or("push config not found")?;
            let re = task.await;
            match re {
                Ok(re) => match re {
                    Ok(_) => {
                        log::debug!(target: "pixiv_bookmarks", "Push task success (task id: {}, index: {}).", self.task.id, i);
                    }
                    Err(e) => {
                        if cfg.allow_failed() {
                            log::warn!(target: "pixiv_bookmarks", "Push task error (task id: {}, index: {}): {}", self.task.id, i, e);
                        } else {
                            log::debug!(target: "pixiv_bookmarks", "Push task error (task id: {}, index: {}): {}", self.task.id, i, e);
                            let e: Result<(), _> = Err(e);
                            concat_pixiv_downloader_error!(error, e);
                        }
                    }
                },
                Err(e) => {
                    if cfg.allow_failed() {
                        log::error!(target: "pixiv_bookmarks", "Push task join error (task id: {}, index: {}): {}", self.task.id, i, e);
                    } else {
                        log::info!(target: "pixiv_bookmarks", "Push task join error (task id: {}, index: {}): {}", self.task.id, i, e);
                        let e: Result<(), _> = Err(e);
                        concat_pixiv_downloader_error!(error, e);
                    }
                }
            }
        }
        error
    }

    pub async fn run(&self) -> Result<(), PixivDownloaderError> {
        let now = chrono::Utc::now();
        if self.send_mode.is_none() {
            match self.ctx.db.get_push_task_data(self.task.id).await? {
                Some(data) => match serde_json::from_str(&data) {
                    Ok(data) => {
                        self.pushed.replace_with2(data);
                    }
                    Err(e) => {
                        log::warn!(target: "pixiv_bookmarks", "Failed to parse push task data: {}", e);
                        log::debug!(target: "pixiv_bookmarks", "Push task data: {}", data);
                    }
                },
                None => {
                    self.first_run.qstore(true);
                }
            }
        }
        if self.use_app_api {
            match self.restrict {
                PixivRestrictType::Public => {
                    self.app_run(PixivRestrictLessType::Public).await?;
                }
                PixivRestrictType::Private => {
                    self.app_run(PixivRestrictLessType::Private).await?;
                }
                PixivRestrictType::All => {
                    self.app_run(PixivRestrictLessType::Public).await?;
                    self.app_run(PixivRestrictLessType::Private).await?;
                }
            }
        } else {
            match self.restrict {
                PixivRestrictType::Public => {
                    self.web_run(false).await?;
                }
                PixivRestrictType::Private => {
                    self.web_run(true).await?;
                }
                PixivRestrictType::All => {
                    self.web_run(false).await?;
                    self.web_run(true).await?;
                }
            }
        }
        if self.send_mode.is_none() {
            let len = self.pushed.get_ref().len();
            if len > self.config.max_len {
                self.pushed.get_mut().drain(0..(len - self.config.max_len));
            }
            let data = serde_json::to_string(self.pushed.get_ref().as_slice())?;
            self.ctx.db.set_push_task_data(self.task.id, &data).await?;
            self.ctx
                .db
                .update_push_task_last_updated(self.task.id, &now)
                .await?;
        }
        Ok(())
    }

    pub async fn web_run(&self, is_hide: bool) -> Result<(), PixivDownloaderError> {
        let pw = self.ctx.pixiv_web_client().await;
        let data = pw
            .get_user_bookmarks(self.uid, is_hide, self.tag, None, None)
            .await
            .ok_or("get user bookmarks failed")?;
        let illusts = &data["works"];
        match self.send_mode {
            Some(m) => {
                if m.is_all() {
                    for i in illusts.members() {
                        self.web_illust(i).await?;
                    }
                } else {
                    if let Some(index) = m.to_index(illusts.len()) {
                        self.web_illust(&illusts[index]).await?;
                    }
                }
            }
            None => {
                if self.first_run.qload() {
                    for i in illusts.members().rev() {
                        if let Some(id) = parse_pixiv_id(&i["id"]) {
                            self.pushed.get_mut().push(id);
                        }
                    }
                } else {
                    for i in illusts.members().rev() {
                        self.web_illust(i).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn web_illust(&self, illust: &JsonValue) -> Result<(), PixivDownloaderError> {
        let id = parse_pixiv_id(&illust["id"]).ok_or("illust id is none")?;
        if self.send_mode.is_none() && self.pushed.get_ref().contains(&id) {
            return Ok(());
        }
        let wdata = match self.data.get_web_data(id) {
            Some(d) => d,
            None => {
                let pw = self.ctx.pixiv_web_client().await;
                let wdata = if self.use_webpage {
                    pw.get_artwork(id).await.ok_or("Failed to get artwork.")?["illust"]
                        [format!("{}", id)]
                    .clone()
                } else {
                    pw.get_artwork_ajax(id)
                        .await
                        .ok_or("Failed to get artwork ajax.")?
                };
                self.data.set_web_data(id, wdata.clone());
                wdata
            }
        };
        let len = illust["pageCount"].as_u64().unwrap_or(1);
        let pdata = if len != 1 {
            let pw = self.ctx.pixiv_web_client().await;
            let pdata = pw
                .get_illust_pages(id)
                .await
                .ok_or("Failed to get illust pages.")?;
            Some(Arc::new(pdata))
        } else {
            None
        };
        if self.send_mode.is_none() {
            self.pushed.get_mut().push(id);
        }
        let wdata = Arc::new(wdata);
        let illust = Arc::new(illust.clone());
        let mut index = 0;
        for i in self.task.push_configs.iter() {
            let cfg = Arc::new(i.clone());
            self.push_manager
                .add_task(
                    index,
                    send_message(
                        self.ctx.clone(),
                        None,
                        Some(wdata.clone()),
                        pdata.clone(),
                        Some(illust.clone()),
                        None,
                        cfg,
                    ),
                    true,
                )
                .await;
            index += 1;
        }
        self.push_manager.join().await;
        self.handle_finished_tasks().await?;
        Ok(())
    }

    pub async fn app_run(
        &self,
        restrict: PixivRestrictLessType,
    ) -> Result<(), PixivDownloaderError> {
        let app = self.ctx.pixiv_app_client().await;
        let app_data = app
            .get_user_bookmarks(self.uid, &restrict, self.tag)
            .await?;
        match self.send_mode {
            Some(m) => {
                if m.is_all() {
                    for i in app_data.illusts.iter() {
                        self.app_illust(i).await?;
                    }
                } else {
                    let index = m.to_index(app_data.illusts.len());
                    if let Some(index) = index {
                        self.app_illust(&app_data.illusts[index]).await?;
                    }
                }
            }
            None => {
                if self.first_run.qload() {
                    for i in app_data.illusts.iter().rev() {
                        if let Some(id) = i.id() {
                            self.pushed.get_mut().push(id);
                        }
                    }
                } else {
                    for i in app_data.illusts.iter().rev() {
                        self.app_illust(i).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn app_illust(&self, illust: &PixivAppIllust) -> Result<(), PixivDownloaderError> {
        let id = illust.id().ok_or("illust id is none")?;
        if self.send_mode.is_none() && self.pushed.get_ref().contains(&id) {
            return Ok(());
        }
        let data = match self.data.get_web_data(id) {
            Some(d) => Some(Arc::new(d)),
            None => {
                if self.use_web_description && illust.caption_is_empty() {
                    let pw = self.ctx.pixiv_web_client().await;
                    match pw.get_artwork_ajax(id).await {
                        Some(data) => {
                            self.data.set_web_data(id, data.clone());
                            Some(Arc::new(data))
                        }
                        None => None,
                    }
                } else {
                    None
                }
            }
        };
        if self.send_mode.is_none() {
            self.pushed.get_mut().push(id);
        }
        let illust = Arc::new(illust.clone());
        let mut index = 0;
        for i in self.task.push_configs.iter() {
            let i = Arc::new(i.clone());
            self.push_manager
                .add_task(
                    index,
                    send_message(
                        self.ctx.clone(),
                        Some(illust.clone()),
                        data.clone(),
                        None,
                        None,
                        None,
                        i,
                    ),
                    true,
                )
                .await;
            index += 1;
        }
        self.push_manager.join().await;
        self.handle_finished_tasks().await?;
        Ok(())
    }
}

pub async fn run_push_task(
    ctx: Arc<ServerContext>,
    task: Arc<PushTask>,
    config: &PushTaskPixivConfig,
    uid: u64,
    restrict: &PixivRestrictType,
    tag: Option<&str>,
    send_mode: Option<&TestSendMode>,
) -> Result<(), PixivDownloaderError> {
    let ctx = RunContext::new(ctx, task, config, uid, restrict, tag, send_mode);
    ctx.run().await
}
