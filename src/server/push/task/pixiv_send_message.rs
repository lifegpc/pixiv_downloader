use super::super::super::preclude::*;
use crate::db::push_task::{AuthorLocation, EveryPushConfig, PushConfig};
use crate::error::PixivDownloaderError;
use crate::opt::author_name_filter::AuthorFiler;
use crate::parser::description::DescriptionParser;
use crate::pixivapp::illust::PixivAppIllust;
use crate::push::every_push::{EveryPushClient, EveryPushTextType};
use crate::{get_helper, gettext};
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

    pub fn id(&self) -> Option<u64> {
        match self.illust {
            Some(i) => match i.id() {
                Some(i) => return Some(i),
                None => {}
            },
            None => {}
        }
        match self.data {
            Some(d) => d["illustId"]
                .as_u64()
                .or_else(|| match d["illustId"].as_str() {
                    Some(s) => s.parse().ok(),
                    None => None,
                }),
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

    pub fn user_id(&self) -> Option<u64> {
        match self.illust {
            Some(i) => match i.user_id() {
                Some(u) => return Some(u),
                None => {}
            },
            None => {}
        }
        match self.data {
            Some(d) => d["userId"].as_u64().or_else(|| match d["userId"].as_str() {
                Some(s) => s.parse().ok(),
                None => None,
            }),
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

    pub fn add_author<S: AsRef<str> + ?Sized>(&self, text: &mut String, author: &S) {
        text.push_str(gettext("by "));
        let author = author.as_ref();
        if let Some(uid) = self.user_id() {
            text.push_str(&format!(
                "[{}](https://www.pixiv.net/users/{})",
                author, uid
            ));
        } else {
            text.push_str(author);
        }
        text.push_str("  \n");
    }

    pub fn desc(&self) -> Option<&str> {
        match self.illust {
            Some(i) => {
                if !i.caption_is_empty() {
                    return i.caption();
                }
            }
            None => {}
        }
        match self.data {
            Some(d) => d["description"]
                .as_str()
                .or_else(|| d["illustComment"].as_str()),
            None => None,
        }
    }

    pub async fn send_every_push(&self, cfg: &EveryPushConfig) -> Result<(), PixivDownloaderError> {
        let client = EveryPushClient::new(&cfg.push_server);
        match cfg.typ {
            EveryPushTextType::Text => {
                let mut title = self.title().map(|s| s.to_owned());
                let author = self.author();
                if cfg.author_locations.contains(&AuthorLocation::Title) {
                    if let Some(t) = &title {
                        if let Some(a) = &author {
                            title = Some(format!("{} - {}", t, a));
                        }
                    }
                }
                let mut text = String::new();
                if cfg.author_locations.contains(&AuthorLocation::Top) {
                    if let Some(a) = &author {
                        text.push_str(gettext("by "));
                        text.push_str(a);
                        text.push_str("\n");
                    }
                }
                if cfg.add_link {
                    if let Some(id) = self.id() {
                        text.push_str(&format!("https://www.pixiv.net/artworks/{}\n", id));
                    }
                }
                if let Some(desc) = self.desc() {
                    let mut p = DescriptionParser::new(false);
                    p.parse(desc)?;
                    text.push_str(&p.data);
                    text.push_str("\n");
                }
                if cfg.author_locations.contains(&AuthorLocation::Bottom) {
                    if let Some(a) = &author {
                        text.push_str(gettext("by "));
                        text.push_str(a);
                        text.push_str("\n");
                    }
                }
                client
                    .push_message(
                        &cfg.push_token,
                        &text,
                        title.as_ref(),
                        cfg.topic_id.as_ref(),
                        Some(EveryPushTextType::Text),
                    )
                    .await?;
            }
            EveryPushTextType::Markdown => {
                let mut title = self.title().map(|s| s.to_owned());
                let author = self.author();
                if cfg.author_locations.contains(&AuthorLocation::Title) {
                    if let Some(t) = &title {
                        if let Some(a) = &author {
                            title = Some(format!("{} - {}", t, a));
                        }
                    }
                }
                let mut text = String::new();
                let len = self.len().unwrap_or(1);
                for i in 0..len {
                    if let Some(url) = self.get_image_url(i).await? {
                        text.push_str(&format!("![​]({})", url));
                    }
                }
                if cfg.author_locations.contains(&AuthorLocation::Top) {
                    if let Some(a) = &author {
                        self.add_author(&mut text, a);
                    }
                }
                if cfg.add_link {
                    if let Some(id) = self.id() {
                        let link = format!("https://www.pixiv.net/artworks/{}", id);
                        text.push_str(&format!("[{}]({})  \n", link, link));
                    }
                }
                if let Some(desc) = self.desc() {
                    let mut p = DescriptionParser::new(true);
                    p.parse(desc)?;
                    text.push_str(&p.data);
                    if !p.data.ends_with("\n\n") {
                        text.push_str("\n\n");
                    }
                }
                if cfg.author_locations.contains(&AuthorLocation::Bottom) {
                    if let Some(a) = &author {
                        self.add_author(&mut text, a);
                    }
                }
                client
                    .push_message(
                        &cfg.push_token,
                        &text,
                        title.as_ref(),
                        cfg.topic_id.as_ref(),
                        Some(EveryPushTextType::Markdown),
                    )
                    .await?;
            }
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
