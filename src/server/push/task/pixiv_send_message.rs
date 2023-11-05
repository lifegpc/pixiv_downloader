use super::super::super::preclude::*;
use crate::db::push_task::{AuthorLocation, EveryPushConfig, PushConfig, PushDeerConfig};
use crate::error::PixivDownloaderError;
use crate::opt::author_name_filter::AuthorFiler;
use crate::parser::description::DescriptionParser;
use crate::pixivapp::illust::PixivAppIllust;
use crate::push::every_push::{EveryPushClient, EveryPushTextType};
use crate::push::pushdeer::PushdeerClient;
use crate::utils::{get_file_name_from_url, parse_pixiv_id};
use crate::{get_helper, gettext};
use json::JsonValue;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

struct RunContext<'a> {
    ctx: Arc<ServerContext>,
    illust: Option<&'a PixivAppIllust>,
    data: Option<&'a JsonValue>,
    pdata: Option<&'a JsonValue>,
    tdata: Option<&'a JsonValue>,
    translated_table: Option<&'a JsonValue>,
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
            Some(d) => return d["title"].as_str().or_else(|| d["illustTitle"].as_str()),
            None => {}
        }
        match self.tdata {
            Some(d) => d["title"].as_str(),
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
            Some(d) => return parse_pixiv_id(&d["id"]).or_else(|| parse_pixiv_id(&d["illustId"])),
            None => {}
        }
        match self.tdata {
            Some(d) => parse_pixiv_id(&d["id"]),
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
            Some(d) => return d["userName"].as_str(),
            None => {}
        }
        match self.tdata {
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
            Some(d) => return parse_pixiv_id(&d["userId"]),
            None => {}
        }
        match self.tdata {
            Some(d) => parse_pixiv_id(&d["userId"]),
            None => None,
        }
    }

    /// Whether to filter author name
    pub fn filter_author(&self) -> bool {
        match self.cfg {
            PushConfig::EveryPush(e) => e.filter_author,
            PushConfig::PushDeer(e) => e.filter_author,
        }
    }

    pub fn len(&self) -> Option<u64> {
        match self.illust {
            Some(i) => return i.page_count(),
            None => {}
        }
        match self.data {
            Some(d) => return d["pageCount"].as_u64(),
            None => {}
        }
        match self.tdata {
            Some(d) => return d["pageCount"].as_u64(),
            None => {}
        }
        match self.pdata {
            Some(d) => Some(d.len() as u64),
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
        match self.pdata {
            Some(d) => {
                return d[index as usize]["urls"]["original"]
                    .as_str()
                    .map(|s| s.to_owned())
            }
            None => {}
        }
        if index == 0 {
            match self.data {
                Some(d) => return d["urls"]["original"].as_str().map(|s| s.to_owned()),
                None => {}
            }
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

    pub fn is_ai(&self) -> bool {
        match self.illust {
            Some(i) => {
                if let Some(id) = i.illust_ai_type() {
                    if id == 2 {
                        return true;
                    }
                }
            }
            None => {}
        }
        match self.data {
            Some(d) => {
                if let Some(id) = d["aiType"].as_u64() {
                    if id == 2 {
                        return true;
                    }
                }
            }
            None => {}
        }
        match self.tdata {
            Some(d) => {
                if let Some(id) = d["aiType"].as_u64() {
                    if id == 2 {
                        return true;
                    }
                }
            }
            None => {}
        }
        false
    }

    pub fn add_ai_tag(&self) -> bool {
        match self.cfg {
            PushConfig::EveryPush(e) => e.add_ai_tag,
            PushConfig::PushDeer(e) => e.add_ai_tag,
        }
    }

    pub fn add_translated_tag(&self) -> bool {
        match self.cfg {
            PushConfig::EveryPush(e) => e.add_translated_tag,
            PushConfig::PushDeer(e) => e.add_translated_tag,
        }
    }

    pub fn add_tags_md(&self, text: &mut String, ensure_ascii: bool) {
        if self.add_ai_tag() && self.is_ai() {
            text.push_str("#");
            text.push_str(gettext("AI generated"));
            text.push_str(" ");
        }
        if let Some(i) = self.illust {
            for tag in i.tags() {
                if let Some(name) = tag.name() {
                    let encoded = if ensure_ascii {
                        percent_encode(name.as_bytes(), NON_ALPHANUMERIC).to_string()
                    } else {
                        name.to_owned()
                    };
                    text.push_str(&format!(
                        "[#{}](https://www.pixiv.net/tags/{}) ",
                        name, &encoded
                    ));
                    if self.add_translated_tag() {
                        if let Some(t) = tag.translated_name() {
                            text.push_str(&format!(
                                "[#{}](https://www.pixiv.net/tags/{}) ",
                                t, &encoded
                            ));
                        }
                    }
                }
            }
            text.push_str(" \n");
            return;
        }
        if let Some(d) = self.data {
            for tag in d["tags"]["tags"].members() {
                if let Some(name) = &tag["tag"].as_str() {
                    let encoded = if ensure_ascii {
                        percent_encode(name.as_bytes(), NON_ALPHANUMERIC).to_string()
                    } else {
                        name.to_string()
                    };
                    text.push_str(&format!(
                        "[#{}](https://www.pixiv.net/tags/{}) ",
                        name, &encoded
                    ));
                    if self.add_translated_tag() {
                        if let Some(t) = &tag["translation"]["en"].as_str() {
                            text.push_str(&format!(
                                "[#{}](https://www.pixiv.net/tags/{}) ",
                                t, &encoded
                            ));
                        }
                    }
                }
            }
            text.push_str(" \n");
            return;
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
                        if cfg.add_link_to_image {
                            text.push_str("[");
                        }
                        text.push_str(&format!("![​]({})", url));
                        if cfg.add_link_to_image {
                            text.push_str(&format!("]({})", url));
                        }
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
                    while !text.ends_with("\n\n") {
                        text.push_str("\n");
                    }
                    text.push_str(&p.data);
                    if !p.data.ends_with("\n\n") {
                        text.push_str("\n\n");
                    }
                }
                if cfg.add_tags {
                    self.add_tags_md(&mut text, false);
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

    pub async fn send_push_deer(&self, cfg: &PushDeerConfig) -> Result<(), PixivDownloaderError> {
        let client = PushdeerClient::new(&cfg.push_server);
        match &cfg.typ {
            EveryPushTextType::Text => {
                let mut title = self.title().map(|s| {
                    if cfg.add_link_to_title {
                        if let Some(id) = self.id() {
                            format!("[{}](https://www.pixiv.net/artworks/{})", s, id)
                        } else {
                            s.to_owned()
                        }
                    } else {
                        s.to_owned()
                    }
                });
                let author = self.author();
                if cfg.author_locations.contains(&AuthorLocation::Title) {
                    if let Some(t) = &title {
                        if let Some(a) = &author {
                            let au = if let Some(uid) = self.user_id() {
                                format!("[{}](https://www.pixiv.net/users/{})", a, uid)
                            } else {
                                a.to_owned()
                            };
                            title = Some(format!("{} - {}", t, au));
                        }
                    }
                }
                let mut text = String::new();
                if let Some(t) = &title {
                    text.push_str(t);
                    text.push_str("\n\n");
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
                if cfg.add_image_link {
                    let len = self.len().unwrap_or(1);
                    for i in 0..len {
                        if let Some(url) = self.get_image_url(i).await? {
                            let name =
                                get_file_name_from_url(&url).unwrap_or_else(|| format!("{}", i));
                            text.push_str(&format!("[{}]({})  \n", name, url));
                        }
                    }
                }
                if let Some(desc) = self.desc() {
                    let mut p = DescriptionParser::builder(true).ensure_link_ascii().build();
                    p.parse(desc)?;
                    while !text.ends_with("\n\n") {
                        text.push_str("\n");
                    }
                    text.push_str(&p.data);
                    if !p.data.ends_with("\n\n") {
                        text.push_str("\n\n");
                    }
                }
                if cfg.add_tags {
                    self.add_tags_md(&mut text, true);
                }
                if cfg.author_locations.contains(&AuthorLocation::Bottom) {
                    if let Some(a) = &author {
                        self.add_author(&mut text, a);
                    }
                }
                client.push_text_message(&cfg.pushkey, &text).await?;
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
                let title = title.ok_or("title not found.")?;
                let mut text = String::new();
                let len = self.len().unwrap_or(1);
                for i in 0..len {
                    if let Some(url) = self.get_image_url(i).await? {
                        if cfg.add_link_to_image {
                            text.push_str("[");
                        }
                        text.push_str(&format!("![​]({})", url));
                        if cfg.add_link_to_image {
                            text.push_str(&format!("]({})", url));
                        }
                        text.push_str("  \n");
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
                    let mut p = DescriptionParser::builder(true).ensure_link_ascii().build();
                    p.parse(desc)?;
                    while !text.ends_with("\n\n") {
                        text.push_str("\n");
                    }
                    text.push_str(&p.data);
                    if !p.data.ends_with("\n\n") {
                        text.push_str("\n\n");
                    }
                }
                if cfg.add_tags {
                    self.add_tags_md(&mut text, true);
                }
                if cfg.author_locations.contains(&AuthorLocation::Bottom) {
                    if let Some(a) = &author {
                        self.add_author(&mut text, a);
                    }
                }
                client
                    .push_markdown_message(&cfg.pushkey, &title, &text)
                    .await?;
            }
            EveryPushTextType::Image => {
                let url = self.get_image_url(0).await?.ok_or("image url not found.")?;
                client.push_image(&cfg.pushkey, &url).await?;
            }
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), PixivDownloaderError> {
        match self.cfg {
            PushConfig::EveryPush(e) => self.send_every_push(e).await,
            PushConfig::PushDeer(e) => self.send_push_deer(e).await,
        }
    }
}

pub async fn send_message(
    ctx: Arc<ServerContext>,
    illust: Option<&PixivAppIllust>,
    data: Option<&JsonValue>,
    pdata: Option<&JsonValue>,
    tdata: Option<&JsonValue>,
    translated_table: Option<&JsonValue>,
    cfg: &PushConfig,
) -> Result<(), PixivDownloaderError> {
    let ctx = RunContext {
        ctx,
        illust,
        data,
        pdata,
        tdata,
        translated_table,
        cfg,
    };
    ctx.run().await
}
