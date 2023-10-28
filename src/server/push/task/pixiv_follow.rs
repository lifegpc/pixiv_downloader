use super::super::super::preclude::*;
use super::pixiv_send_message::send_message;
use super::TestSendMode;
use crate::db::push_task::PushTaskPixivConfig;
use crate::db::PushTask;
use crate::ext::rw_lock::GetRwLock;
use crate::get_helper;
use crate::pixiv_app::PixivRestrictType;
use crate::pixivapp::illust::PixivAppIllust;
use json::JsonValue;
use std::collections::HashMap;
use std::sync::RwLock;

struct PixivFollowData {
    web_list: RwLock<Option<JsonValue>>,
    web_data: RwLock<HashMap<u64, JsonValue>>,
}

impl PixivFollowData {
    pub fn new() -> Self {
        Self {
            web_list: RwLock::new(None),
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
    task: &'a PushTask,
    config: &'a PushTaskPixivConfig,
    restrict: &'a PixivRestrictType,
    send_mode: Option<&'a TestSendMode>,
    data: Arc<PixivFollowData>,
    use_app_api: bool,
    use_web_description: bool,
    use_webpage: bool,
}

impl<'a> RunContext<'a> {
    pub fn new(
        ctx: Arc<ServerContext>,
        task: &'a PushTask,
        config: &'a PushTaskPixivConfig,
        restrict: &'a PixivRestrictType,
        send_mode: Option<&'a TestSendMode>,
    ) -> Self {
        let helper = get_helper();
        Self {
            ctx,
            task,
            config,
            restrict,
            send_mode,
            data: Arc::new(PixivFollowData::new()),
            use_app_api: config.use_app_api.unwrap_or(helper.use_app_api()),
            use_web_description: config
                .use_web_description
                .unwrap_or(helper.use_web_description()),
            use_webpage: config.use_webpage.unwrap_or(helper.use_webpage()),
        }
    }

    pub async fn run(&self) -> Result<(), PixivDownloaderError> {
        if self.use_app_api {
            self.app_run().await?;
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
            None => {}
        }
        Ok(())
    }

    pub async fn app_illust(&self, illust: &PixivAppIllust) -> Result<(), PixivDownloaderError> {
        let id = illust.id().ok_or("illust id is none")?;
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
        for i in self.task.push_configs.iter() {
            send_message(self.ctx.clone(), Some(&illust), data.as_ref(), i).await?;
        }
        Ok(())
    }
}

pub async fn run_push_task(
    ctx: Arc<ServerContext>,
    task: &PushTask,
    config: &PushTaskPixivConfig,
    restrict: &PixivRestrictType,
    send_mode: Option<&TestSendMode>,
) -> Result<(), PixivDownloaderError> {
    let ctx = RunContext::new(ctx, task, config, restrict, send_mode);
    ctx.run().await
}
