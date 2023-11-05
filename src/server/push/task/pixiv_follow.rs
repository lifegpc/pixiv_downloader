use super::super::super::preclude::*;
use super::pixiv_send_message::send_message;
use super::TestSendMode;
use crate::db::push_task::{PixivMode, PushTaskPixivConfig};
use crate::db::PushTask;
use crate::ext::atomic::AtomicQuick;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::get_helper;
use crate::pixiv_app::PixivRestrictType;
use crate::pixivapp::illust::PixivAppIllust;
use crate::utils::parse_pixiv_id;
use json::JsonValue;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::RwLock;

struct PixivFollowData {
    web_data: RwLock<HashMap<u64, JsonValue>>,
}

impl PixivFollowData {
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
    restrict: &'a PixivRestrictType,
    mode: &'a PixivMode,
    send_mode: Option<&'a TestSendMode>,
    data: Arc<PixivFollowData>,
    use_app_api: bool,
    use_web_description: bool,
    use_webpage: bool,
    pushed: RwLock<Vec<u64>>,
    first_run: AtomicBool,
}

impl<'a> RunContext<'a> {
    pub fn new(
        ctx: Arc<ServerContext>,
        task: Arc<PushTask>,
        config: &'a PushTaskPixivConfig,
        restrict: &'a PixivRestrictType,
        mode: &'a PixivMode,
        send_mode: Option<&'a TestSendMode>,
    ) -> Self {
        let helper = get_helper();
        Self {
            ctx,
            task,
            config,
            restrict,
            mode,
            send_mode,
            data: Arc::new(PixivFollowData::new()),
            use_app_api: config.use_app_api.unwrap_or(helper.use_app_api()),
            use_web_description: config
                .use_web_description
                .unwrap_or(helper.use_web_description()),
            use_webpage: config.use_webpage.unwrap_or(helper.use_webpage()),
            pushed: RwLock::new(Vec::new()),
            first_run: AtomicBool::new(false),
        }
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
                        log::warn!(target: "pixiv_follow", "Failed to parse push task data: {}", e);
                        log::debug!(target: "pixiv_follow", "Push task data: {}", data);
                    }
                },
                None => {
                    self.first_run.qstore(true);
                }
            }
        }
        if self.use_app_api {
            self.app_run().await?;
        } else {
            self.web_run().await?;
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

    pub async fn web_run(&self) -> Result<(), PixivDownloaderError> {
        let pw = self.ctx.pixiv_web_client().await;
        let data = pw
            .get_follow(1, self.mode.is_r18())
            .await
            .ok_or("Failed to get follow.")?;
        let illusts = &data["thumbnails"]["illust"];
        match self.send_mode {
            Some(m) => {
                if m.is_all() {
                    for i in illusts.members() {
                        self.web_illust(i, &data).await?;
                    }
                } else {
                    let index = m.to_index(illusts.len());
                    if let Some(index) = index {
                        self.web_illust(&illusts[index], &data).await?;
                    }
                }
            }
            None => {
                if self.first_run.qload() {
                    for i in illusts.members() {
                        if let Some(id) = parse_pixiv_id(&i["id"]) {
                            self.pushed.get_mut().push(id);
                        }
                    }
                } else {
                    for i in illusts.members() {
                        self.web_illust(i, &data).await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn web_illust(
        &self,
        illust: &JsonValue,
        data: &JsonValue,
    ) -> Result<(), PixivDownloaderError> {
        let id = parse_pixiv_id(&illust["id"]).ok_or("illust id is none")?;
        if self.send_mode.is_none() && self.pushed.get_ref().contains(&id) {
            return Ok(());
        }
        let wdata = match self.data.get_web_data(id) {
            Some(d) => d,
            None => {
                let pw = self.ctx.pixiv_web_client().await;
                let wdata = pw
                    .get_artwork_ajax(id)
                    .await
                    .ok_or("Failed to get artwork ajax.")?;
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
            Some(pdata)
        } else {
            None
        };
        if self.send_mode.is_none() {
            self.pushed.get_mut().push(id);
        }
        for i in self.task.push_configs.iter() {
            send_message(
                self.ctx.clone(),
                None,
                Some(&wdata),
                pdata.as_ref(),
                Some(illust),
                Some(&data["tagTranslation"]),
                i,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn app_run(&self) -> Result<(), PixivDownloaderError> {
        let app = self.ctx.pixiv_app_client().await;
        let app_data = app.get_follow(self.restrict).await?;
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
                    for i in app_data.illusts.iter() {
                        if let Some(id) = i.id() {
                            self.pushed.get_mut().push(id);
                        }
                    }
                } else {
                    for i in app_data.illusts.iter() {
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
            Some(d) => Some(d),
            None => {
                if self.use_web_description && illust.caption_is_empty() {
                    let pw = self.ctx.pixiv_web_client().await;
                    match pw.get_artwork_ajax(id).await {
                        Some(data) => {
                            self.data.set_web_data(id, data.clone());
                            Some(data)
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
        for i in self.task.push_configs.iter() {
            send_message(
                self.ctx.clone(),
                Some(&illust),
                data.as_ref(),
                None,
                None,
                None,
                i,
            )
            .await?;
        }
        Ok(())
    }
}

pub async fn run_push_task(
    ctx: Arc<ServerContext>,
    task: Arc<PushTask>,
    config: &PushTaskPixivConfig,
    restrict: &PixivRestrictType,
    mode: &PixivMode,
    send_mode: Option<&TestSendMode>,
) -> Result<(), PixivDownloaderError> {
    let ctx = RunContext::new(ctx, task, config, restrict, mode, send_mode);
    ctx.run().await
}
