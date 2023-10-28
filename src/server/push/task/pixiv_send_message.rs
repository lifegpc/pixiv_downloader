use super::super::super::preclude::*;
use crate::db::push_task::{AuthorLocation, EveryPushConfig, PushConfig};
use crate::error::PixivDownloaderError;
use crate::get_helper;
use crate::opt::author_name_filter::AuthorFiler;
use crate::pixivapp::illust::PixivAppIllust;
use crate::push::every_push::{EveryPushClient, EveryPushTextType};
use json::JsonValue;

struct RunContext<'a> {
    ctx: Arc<ServerContext>,
    illust: Option<&'a PixivAppIllust>,
    data: Option<&'a JsonValue>,
    cfg: &'a PushConfig,
}

impl<'a> RunContext<'a> {
    pub fn title(&self) -> Option<&str> {
        match self.illust {
            Some(i) => match i.title() {
                Some(t) => return Some(t),
                None => {}
            },
            None => {}
        }
        match self.data {
            Some(d) => d["illustTitle"].as_str(),
            None => None,
        }
    }

    pub fn _author(&self) -> Option<&str> {
        match self.illust {
            Some(i) => match i.user_name() {
                Some(u) => return Some(u),
                None => {}
            },
            None => {}
        }
        match self.data {
            Some(d) => d["userName"].as_str(),
            None => None,
        }
    }

    pub fn author(&self) -> Option<String> {
        match self._author() {
            Some(a) => {
                if self.filter_author() {
                    match get_helper().author_name_filters() {
                        Some(l) => return Some(l.filter(a)),
                        None => {}
                    }
                }
                Some(a.to_owned())
            }
            None => None,
        }
    }

    /// Whether to filter author name
    pub fn filter_author(&self) -> bool {
        match self.cfg {
            PushConfig::EveryPush(e) => e.filter_author,
        }
    }

    pub fn len(&self) -> Option<u64> {
        match self.illust {
            Some(i) => return i.page_count(),
            None => {}
        }
        match self.data {
            Some(d) => d["pageCount"].as_u64(),
            None => None,
        }
    }

    pub fn _get_image_url(&self, index: u64) -> Option<String> {
        let len = self.len().unwrap_or(1);
        if index >= len {
            return None;
        }
        if len == 1 {
            match self.illust {
                Some(i) => return i.original_image_url().map(|s| s.to_owned()),
                None => {}
            }
        }
        match self.illust {
            Some(i) => match i.meta_pages().get(index as usize) {
                Some(p) => return p.original().map(|s| s.to_owned()),
                None => {}
            },
            None => {}
        }
        None
    }

    pub async fn get_image_url(&self, index: u64) -> Result<Option<String>, PixivDownloaderError> {
        match self._get_image_url(index) {
            Some(u) => Ok(Some(self.ctx.generate_pixiv_proxy_url(u).await?)),
            None => Ok(None),
        }
    }

    pub async fn send_every_push(&self, cfg: &EveryPushConfig) -> Result<(), PixivDownloaderError> {
        let client = EveryPushClient::new(&cfg.push_server);
        match cfg.typ {
            EveryPushTextType::Text => {}
            EveryPushTextType::Markdown => {}
            EveryPushTextType::Image => {
                let mut title = self.title().map(|s| s.to_owned());
                if cfg.author_locations.contains(&AuthorLocation::Title) {
                    if let Some(t) = &title {
                        if let Some(a) = self.author() {
                            title = Some(format!("{} - {}", t, a));
                        }
                    }
                }
                let url = self.get_image_url(0).await?.ok_or("image url not found.")?;
                client
                    .push_message(
                        &cfg.push_token,
                        &url,
                        title.as_ref(),
                        cfg.topic_id.as_ref(),
                        Some(EveryPushTextType::Image),
                    )
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn run(&self) -> Result<(), PixivDownloaderError> {
        match self.cfg {
            PushConfig::EveryPush(e) => self.send_every_push(e).await,
        }
    }
}

pub async fn send_message(
    ctx: Arc<ServerContext>,
    illust: Option<&PixivAppIllust>,
    data: Option<&JsonValue>,
    cfg: &PushConfig,
) -> Result<(), PixivDownloaderError> {
    let ctx = RunContext {
        ctx,
        illust,
        data,
        cfg,
    };
    ctx.run().await
}
