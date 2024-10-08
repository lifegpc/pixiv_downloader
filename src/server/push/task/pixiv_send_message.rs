use super::super::super::preclude::*;
use crate::db::push_task::{
    AuthorLocation, EveryPushConfig, PushConfig, PushDeerConfig, TelegramBackend,
    TelegramBigPhotoSendMethod, TelegramPushConfig,
};
use crate::error::PixivDownloaderError;
use crate::formdata::FormDataPartBuilder;
use crate::opt::author_name_filter::AuthorFiler;
use crate::parser::description::convert_description_to_tg_html;
use crate::parser::description::DescriptionParser;
use crate::pixivapp::illust::PixivAppIllust;
use crate::push::every_push::{EveryPushClient, EveryPushTextType};
use crate::push::pushdeer::PushdeerClient;
use crate::push::telegram::botapi_client::{BotapiClient, BotapiClientConfig};
use crate::push::telegram::image::{
    generate_image, get_thumbnail_filename, is_supported_image, MAX_PHOTO_SIZE,
};
use crate::push::telegram::text::{encode_data, TextSpliter};
use crate::push::telegram::tg_type::{
    InputFile, InputMedia, InputMediaDocumentBuilder, InputMediaPhotoBuilder, ParseMode,
    ReplyParametersBuilder,
};
use crate::utils::{get_file_name_from_url, parse_pixiv_id};
use crate::{get_helper, gettext};
use json::JsonValue;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use regex::Regex;

lazy_static! {
    #[doc(hidden)]
    static ref RE: Regex = Regex::new("[,\\./\\\\;'\\[\\]\\{\\}!@#\\$%\\^&\\*\\(\\)\\-= \\+<>\\?\\:\"]").unwrap();
}

struct RunContext {
    ctx: Arc<ServerContext>,
    illust: Option<Arc<PixivAppIllust>>,
    data: Option<Arc<JsonValue>>,
    pdata: Option<Arc<JsonValue>>,
    tdata: Option<Arc<JsonValue>>,
    translated_table: Option<Arc<JsonValue>>,
    cfg: Arc<PushConfig>,
}

impl RunContext {
    pub fn title(&self) -> Option<&str> {
        match self.illust.as_ref() {
            Some(i) => match i.title() {
                Some(t) => return Some(t),
                None => {}
            },
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => return d["title"].as_str().or_else(|| d["illustTitle"].as_str()),
            None => {}
        }
        match self.tdata.as_ref() {
            Some(d) => d["title"].as_str(),
            None => None,
        }
    }

    pub fn id(&self) -> Option<u64> {
        match self.illust.as_ref() {
            Some(i) => match i.id() {
                Some(i) => return Some(i),
                None => {}
            },
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => return parse_pixiv_id(&d["id"]).or_else(|| parse_pixiv_id(&d["illustId"])),
            None => {}
        }
        match self.tdata.as_ref() {
            Some(d) => parse_pixiv_id(&d["id"]),
            None => None,
        }
    }

    pub fn _author(&self) -> Option<&str> {
        match self.illust.as_ref() {
            Some(i) => match i.user_name() {
                Some(u) => return Some(u),
                None => {}
            },
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => return d["userName"].as_str(),
            None => {}
        }
        match self.tdata.as_ref() {
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
        match self.illust.as_ref() {
            Some(i) => match i.user_id() {
                Some(u) => return Some(u),
                None => {}
            },
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => return parse_pixiv_id(&d["userId"]),
            None => {}
        }
        match self.tdata.as_ref() {
            Some(d) => parse_pixiv_id(&d["userId"]),
            None => None,
        }
    }

    /// Whether to filter author name
    pub fn filter_author(&self) -> bool {
        match self.cfg.as_ref() {
            PushConfig::EveryPush(e) => e.filter_author,
            PushConfig::PushDeer(e) => e.filter_author,
            PushConfig::Telegram(e) => e.filter_author,
        }
    }

    pub fn len(&self) -> Option<u64> {
        match self.illust.as_ref() {
            Some(i) => return i.page_count(),
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => return d["pageCount"].as_u64(),
            None => {}
        }
        match self.tdata.as_ref() {
            Some(d) => return d["pageCount"].as_u64(),
            None => {}
        }
        match self.pdata.as_ref() {
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
            match self.illust.as_ref() {
                Some(i) => return i.original_image_url().map(|s| s.to_owned()),
                None => {}
            }
        }
        match self.illust.as_ref() {
            Some(i) => match i.meta_pages().get(index as usize) {
                Some(p) => return p.original().map(|s| s.to_owned()),
                None => {}
            },
            None => {}
        }
        match self.pdata.as_ref() {
            Some(d) => {
                return d[index as usize]["urls"]["original"]
                    .as_str()
                    .map(|s| s.to_owned())
            }
            None => {}
        }
        if index == 0 {
            match self.data.as_ref() {
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

    pub async fn get_input_file(
        &self,
        index: u64,
        download_media: bool,
        cfg: &TelegramPushConfig,
    ) -> Result<Option<(InputFile, bool)>, PixivDownloaderError> {
        if download_media {
            match self._get_image_url(index) {
                Some(u) => match self
                    .ctx
                    .tmp_cache
                    .get_cache(&u, json::object! { "referer": "https://www.pixiv.net/" })
                    .await
                {
                    Ok(p) => {
                        let (is_supported, too_big) = match is_supported_image(&p).await {
                            Ok(s) => (s.supported, s.size_too_big),
                            Err(e) => {
                                log::warn!(target: "pixiv_send_message", "Failed to test image is supported by using ffprobe: {}", e);
                                let meta = tokio::fs::metadata(&p).await?;
                                if meta.len() >= MAX_PHOTO_SIZE {
                                    (false, false)
                                } else {
                                    (true, false)
                                }
                            }
                        };
                        let send_as_file =
                            !is_supported && (!too_big || cfg.big_photo.is_document());
                        let p = if !is_supported && !send_as_file {
                            match &cfg.big_photo {
                                TelegramBigPhotoSendMethod::Compress(c) => {
                                    if let Ok(filename) =
                                        get_thumbnail_filename(&p, c.max_side, c.quality)
                                    {
                                        let fn1 = filename.to_string_lossy();
                                        let o = match self.ctx.tmp_cache.get_local_cache(&fn1).await
                                        {
                                            Ok(o) => o,
                                            Err(_) => None,
                                        };
                                        match o {
                                            Some(o) => o,
                                            None => {
                                                match generate_image(
                                                    &p, &filename, c.max_side, c.quality,
                                                )
                                                .await
                                                {
                                                    Ok(_) => {
                                                        let _ = self
                                                            .ctx
                                                            .tmp_cache
                                                            .push_local_cache(&fn1)
                                                            .await;
                                                        filename
                                                    }
                                                    Err(e) => {
                                                        log::warn!(target: "pixiv_send_message", "Failed to generate thumbnial: {}", e);
                                                        p
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        p
                                    }
                                }
                                TelegramBigPhotoSendMethod::Document => p,
                            }
                        } else {
                            p
                        };
                        let name = p
                            .file_name()
                            .map(|a| a.to_str().unwrap_or(""))
                            .unwrap_or("")
                            .to_owned();
                        let f = FormDataPartBuilder::default()
                            .body(p)
                            .filename(name)
                            .build()
                            .map_err(|_| "Failed to create FormDataPart.")?;
                        Ok(Some((InputFile::Content(f), send_as_file)))
                    }
                    Err(e) => Err(e),
                },
                None => Ok(None),
            }
        } else {
            match self.get_image_url(index).await {
                Ok(Some(u)) => Ok(Some((InputFile::URL(u), false))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
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
        match self.illust.as_ref() {
            Some(i) => {
                if !i.caption_is_empty() {
                    return i.caption();
                }
            }
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => d["description"]
                .as_str()
                .or_else(|| d["illustComment"].as_str()),
            None => None,
        }
    }

    pub fn is_ai(&self) -> bool {
        match self.illust.as_ref() {
            Some(i) => {
                if let Some(id) = i.illust_ai_type() {
                    if id == 2 {
                        return true;
                    }
                }
            }
            None => {}
        }
        match self.data.as_ref() {
            Some(d) => {
                if let Some(id) = d["aiType"].as_u64() {
                    if id == 2 {
                        return true;
                    }
                }
            }
            None => {}
        }
        match self.tdata.as_ref() {
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

    pub fn is_r18(&self) -> bool {
        if let Some(i) = self.illust.as_ref() {
            for tag in i.tags() {
                if let Some(name) = tag.name() {
                    if name == "R-18" {
                        return true;
                    }
                }
            }
        }
        if let Some(d) = self.data.as_ref() {
            for tag in d["tags"]["tags"].members() {
                if let Some(name) = &tag["tag"].as_str() {
                    if *name == "R-18" {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn add_ai_tag(&self) -> bool {
        match self.cfg.as_ref() {
            PushConfig::EveryPush(e) => e.add_ai_tag,
            PushConfig::PushDeer(e) => e.add_ai_tag,
            PushConfig::Telegram(e) => e.add_ai_tag,
        }
    }

    pub fn add_translated_tag(&self) -> bool {
        match self.cfg.as_ref() {
            PushConfig::EveryPush(e) => e.add_translated_tag,
            PushConfig::PushDeer(e) => e.add_translated_tag,
            PushConfig::Telegram(e) => e.add_translated_tag,
        }
    }

    pub fn add_tags_md(&self, text: &mut String, ensure_ascii: bool) {
        if self.add_ai_tag() && self.is_ai() {
            text.push_str("#");
            text.push_str(gettext("AI generated"));
            text.push_str(" ");
        }
        if let Some(i) = self.illust.as_ref() {
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
        if let Some(d) = self.data.as_ref() {
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

    pub fn add_tags_tg(&self, text: &mut String, cfg: &TelegramPushConfig) {
        if cfg.add_ai_tag && self.is_ai() {
            text.push_str("#");
            text.push_str(&gettext("AI generated").replace(' ', "_"));
            text.push_str(" ");
        }
        if let Some(i) = self.illust.as_ref() {
            for tag in i.tags() {
                if let Some(name) = tag.name() {
                    if cfg.add_link_to_tag {
                        text.push_str(&format!(
                            "<a href=\"https://www.pixiv.net/tags/{}\">#{}</a> ",
                            name, name
                        ));
                    } else {
                        text.push('#');
                        text.push_str(&RE.replace_all(name, "_").into_owned());
                        text.push(' ');
                    }
                    if cfg.add_translated_tag {
                        if let Some(t) = tag.translated_name() {
                            if cfg.add_link_to_tag {
                                text.push_str(&format!(
                                    "<a href=\"https://www.pixiv.net/tags/{}\">#{}</a> ",
                                    name, t
                                ));
                            } else {
                                text.push('#');
                                text.push_str(&RE.replace_all(t, "_").into_owned());
                                text.push(' ');
                            }
                        }
                    }
                }
            }
            text.push_str("\n");
            return;
        }
        if let Some(d) = self.data.as_ref() {
            for tag in d["tags"]["tags"].members() {
                if let Some(name) = &tag["tag"].as_str() {
                    if cfg.add_link_to_tag {
                        text.push_str(&format!(
                            "<a href=\"https://www.pixiv.net/tags/{}\">#{}</a> ",
                            name, name
                        ));
                    } else {
                        text.push('#');
                        text.push_str(&RE.replace_all(name, "_").into_owned());
                        text.push(' ');
                    }
                    if self.add_translated_tag() {
                        if let Some(t) = &tag["translation"]["en"].as_str() {
                            if cfg.add_link_to_tag {
                                text.push_str(&format!(
                                    "<a href=\"https://www.pixiv.net/tags/{}\">#{}</a> ",
                                    name, t
                                ));
                            } else {
                                text.push('#');
                                text.push_str(&RE.replace_all(t, "_").into_owned());
                                text.push(' ');
                            }
                        }
                    }
                }
            }
            text.push_str("\n");
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
                    let mut p = DescriptionParser::new(false, false);
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
                    let mut p = DescriptionParser::new(true, false);
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
                    let mut p = DescriptionParser::builder(true, false)
                        .ensure_link_ascii()
                        .build();
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
                    let mut p = DescriptionParser::builder(true, false)
                        .ensure_link_ascii()
                        .build();
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

    pub async fn send_telegram(
        &self,
        cfg: &TelegramPushConfig,
    ) -> Result<(), PixivDownloaderError> {
        match &cfg.backend {
            TelegramBackend::Botapi(b) => self.send_telegram_botapi(cfg, b).await,
        }
    }

    pub async fn send_telegram_botapi(
        &self,
        cfg: &TelegramPushConfig,
        b: &BotapiClientConfig,
    ) -> Result<(), PixivDownloaderError> {
        let c = BotapiClient::new(b);
        let is_r18 = self.is_r18();
        let mut text = String::new();
        let mut title = self.title().unwrap_or("");
        if title.is_empty() {
            title = "Unknown title";
        }
        if cfg.add_link_to_title {
            if let Some(id) = self.id() {
                text += &format!(
                    "<a href=\"https://www.pixiv.net/artworks/{}\">{}</a>",
                    id,
                    encode_data(title)
                );
            } else {
                text += &encode_data(title);
            }
        } else {
            text += &encode_data(title);
        }
        let author = self.author().map(|a| {
            if let Some(uid) = self.user_id() {
                format!(
                    "<a href=\"https://www.pixiv.net/users/{}\">{}</a>",
                    uid,
                    encode_data(&a)
                )
            } else {
                encode_data(&a)
            }
        });
        if cfg.author_locations.contains(&AuthorLocation::Title) {
            if let Some(author) = &author {
                text += " - ";
                text += author;
            }
        }
        text += "\n";
        if cfg.author_locations.contains(&AuthorLocation::Top) {
            if let Some(a) = &author {
                text.push_str(gettext("by "));
                text += a;
                text.push('\n');
            }
        }
        if cfg.add_link {
            if let Some(id) = self.id() {
                text += &format!("https://www.pixiv.net/artworks/{}\n", id);
            }
        }
        text += &convert_description_to_tg_html(self.desc().unwrap_or(""))?;
        text.push('\n');
        if cfg.add_tags {
            self.add_tags_tg(&mut text, cfg);
        }
        if cfg.author_locations.contains(&AuthorLocation::Bottom) {
            if let Some(a) = &author {
                text.push_str(gettext("by "));
                text += a;
                text.push('\n');
            }
        }
        let mut ts = TextSpliter::builder().max_length(1024).build();
        ts.parse(&text)?;
        let len = self.len().unwrap_or(1);
        let mut last_message_id: Option<i64> = None;
        let download_media = cfg.download_media.unwrap_or(c.is_custom());
        if len == 1 {
            let (f, send_as_file) = self
                .get_input_file(0, download_media, cfg)
                .await?
                .ok_or("Failed to get image.")?;
            let r = match last_message_id {
                Some(m) => Some(
                    ReplyParametersBuilder::default()
                        .message_id(m)
                        .build()
                        .map_err(|_| "Failed to gen.")?,
                ),
                None => None,
            };
            let text = ts.to_html(None);
            let m = if send_as_file {
                c.send_document(
                    &cfg.chat_id,
                    cfg.message_thread_id,
                    f,
                    None,
                    Some(text.as_str()),
                    Some(ParseMode::HTML),
                    None,
                    Some(cfg.disable_notification),
                    Some(cfg.protect_content),
                    None,
                    r.as_ref(),
                )
                .await?
                .to_result()?
            } else {
                c.send_photo(
                    &cfg.chat_id,
                    cfg.message_thread_id,
                    f,
                    Some(text.as_str()),
                    Some(ParseMode::HTML),
                    Some(cfg.show_caption_above_media),
                    Some(is_r18),
                    Some(cfg.disable_notification),
                    Some(cfg.protect_content),
                    None,
                    r.as_ref(),
                )
                .await?
                .to_result()?
            };
            last_message_id = Some(m.message_id);
        } else {
            let mut i = 0u64;
            let mut photos = Vec::new();
            let mut photo_files = Vec::new();
            let mut new_photos = Vec::new();
            let mut new_photo_files = Vec::new();
            let mut have_doc = false;
            let mut have_nondoc = false;
            let mut new_have_doc = false;
            let mut new_have_nondoc = false;
            while i < len {
                let (f, send_as_file) = self
                    .get_input_file(i, download_media, cfg)
                    .await?
                    .ok_or("Failed to get image.")?;
                let mut is_content = false;
                let u = match f {
                    InputFile::URL(u) => u,
                    InputFile::Content(c) => {
                        photo_files.push((format!("img{}", i), c));
                        is_content = true;
                        format!("attach://img{}", i)
                    }
                };
                if send_as_file {
                    let mut doc = InputMediaDocumentBuilder::default();
                    doc.media(u);
                    if photos.is_empty() {
                        let text = ts.to_html(None);
                        doc.caption(Some(text)).parse_mode(Some(ParseMode::HTML));
                    }
                    let doc = doc.build().map_err(|_| "Failed to gen.")?;
                    if have_nondoc {
                        new_photos.push(InputMedia::from(doc));
                        if is_content {
                            match photo_files.pop() {
                                Some(p) => new_photo_files.push(p),
                                None => {}
                            }
                        }
                        new_have_doc = true;
                    } else {
                        photos.push(InputMedia::from(doc));
                        have_doc = true;
                    }
                } else {
                    let mut img = InputMediaPhotoBuilder::default();
                    img.media(u).has_spoiler(is_r18);
                    if photos.is_empty() {
                        let text = ts.to_html(None);
                        img.caption(Some(text)).parse_mode(Some(ParseMode::HTML));
                    }
                    let img = img.build().map_err(|_| "Failed to gen.")?;
                    if have_doc {
                        new_photos.push(InputMedia::from(img));
                        if is_content {
                            match photo_files.pop() {
                                Some(p) => new_photo_files.push(p),
                                None => {}
                            }
                        }
                        new_have_nondoc = true;
                    } else {
                        photos.push(InputMedia::from(img));
                        have_nondoc = true;
                    }
                }
                i += 1;
                while (i == len && !photos.is_empty())
                    || photos.len() == 10
                    || !new_photos.is_empty()
                {
                    let r = match last_message_id {
                        Some(m) => Some(
                            ReplyParametersBuilder::default()
                                .message_id(m)
                                .build()
                                .map_err(|_| "Failed to gen.")?,
                        ),
                        None => None,
                    };
                    let m = c
                        .send_media_group(
                            &cfg.chat_id,
                            cfg.message_thread_id,
                            photos,
                            photo_files,
                            Some(cfg.disable_notification),
                            Some(cfg.protect_content),
                            None,
                            r.as_ref(),
                        )
                        .await?
                        .to_result()?;
                    last_message_id = m.first().map(|m| m.message_id);
                    photos = new_photos;
                    photo_files = new_photo_files;
                    new_photos = Vec::new();
                    new_photo_files = Vec::new();
                    have_doc = new_have_doc;
                    have_nondoc = new_have_nondoc;
                    new_have_doc = false;
                    new_have_nondoc = false;
                }
            }
        }
        while !ts.is_empty() {
            let r = match last_message_id {
                Some(m) => Some(
                    ReplyParametersBuilder::default()
                        .message_id(m)
                        .build()
                        .map_err(|_| "Failed to gen.")?,
                ),
                None => None,
            };
            let text = ts.to_html(Some(4096));
            let m = c
                .send_message(
                    &cfg.chat_id,
                    cfg.message_thread_id,
                    &text,
                    Some(ParseMode::HTML),
                    None,
                    Some(cfg.disable_notification),
                    Some(cfg.protect_content),
                    None,
                    r.as_ref(),
                )
                .await?
                .to_result()?;
            last_message_id = Some(m.message_id);
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), PixivDownloaderError> {
        match self.cfg.as_ref() {
            PushConfig::EveryPush(e) => self.send_every_push(e).await,
            PushConfig::PushDeer(e) => self.send_push_deer(e).await,
            PushConfig::Telegram(e) => self.send_telegram(e).await,
        }
    }
}

pub async fn send_message(
    ctx: Arc<ServerContext>,
    illust: Option<Arc<PixivAppIllust>>,
    data: Option<Arc<JsonValue>>,
    pdata: Option<Arc<JsonValue>>,
    tdata: Option<Arc<JsonValue>>,
    translated_table: Option<Arc<JsonValue>>,
    cfg: Arc<PushConfig>,
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
