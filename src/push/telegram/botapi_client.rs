use super::tg_type::*;
use crate::formdata::FormData;
#[cfg(test)]
use crate::formdata::FormDataPartBuilder;
use crate::webclient::WebClient;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_base() -> String {
    String::from("https://api.telegram.org")
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
/// Bot API Client Config
pub struct BotapiClientConfig {
    #[serde(default = "default_base")]
    #[builder(default = "String::from(\"https://api.telegram.org\")")]
    /// Bot API Server. Default: https://api.telegram.org
    pub base: String,
    /// Auth token
    pub token: String,
}

pub struct BotapiClient {
    cfg: BotapiClientConfig,
    client: WebClient,
}

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum BotapiClientError {
    SerdeJson(serde_json::Error),
    Str(String),
}

impl From<&str> for BotapiClientError {
    fn from(value: &str) -> Self {
        Self::Str(value.to_owned())
    }
}

impl BotapiClient {
    pub fn new(cfg: &BotapiClientConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            client: WebClient::default(),
        }
    }

    pub async fn send_animation<T: AsRef<str> + ?Sized>(
        &self,
        chat_id: &ChatId,
        message_thread_id: Option<i64>,
        animation: InputFile,
        duration: Option<i64>,
        width: Option<i64>,
        height: Option<i64>,
        thumbnail: Option<InputFile>,
        caption: Option<&str>,
        parse_mode: Option<ParseMode>,
        show_caption_above_media: Option<bool>,
        has_spoiler: Option<bool>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        message_effect_id: Option<&str>,
        reply_parameters: Option<&ReplyParameters>,
    ) -> Result<BotApiResult<Message>, BotapiClientError> {
        let mut form = FormData::new();
        form.data("chat_id", &chat_id.to_string());
        match message_thread_id {
            Some(m) => {
                form.data("message_thread_id", &m.to_string());
            }
            None => {}
        }
        match animation {
            InputFile::URL(u) => {
                form.data("animation", &u);
            }
            InputFile::Content(c) => {
                form.part("animation", c);
            }
        }
        match duration {
            Some(d) => {
                form.data("duration", &d.to_string());
            }
            None => {}
        }
        match width {
            Some(d) => {
                form.data("width", &d.to_string());
            }
            None => {}
        }
        match height {
            Some(d) => {
                form.data("height", &d.to_string());
            }
            None => {}
        }
        match thumbnail {
            Some(m) => match m {
                InputFile::URL(u) => {
                    form.data("thumbnail", &u);
                }
                InputFile::Content(c) => {
                    form.part("thumbnail", c);
                }
            },
            None => {}
        }
        match caption {
            Some(c) => {
                form.data("caption", c);
            }
            None => {}
        }
        match parse_mode {
            Some(p) => {
                form.data("parse_mode", p.as_ref());
            }
            None => {}
        }
        match show_caption_above_media {
            Some(p) => {
                form.data("show_caption_above_media", &p.to_string());
            }
            None => {}
        }
        match has_spoiler {
            Some(p) => {
                form.data("has_spoiler", &p.to_string());
            }
            None => {}
        }
        match disable_notification {
            Some(d) => {
                form.data("disable_notification", &d.to_string());
            }
            None => {}
        }
        match protect_content {
            Some(p) => {
                form.data("protect_content", &p.to_string());
            }
            None => {}
        }
        match message_effect_id {
            Some(m) => {
                form.data("message_effect_id", m);
            }
            None => {}
        }
        match reply_parameters {
            Some(r) => {
                form.data("reply_parameters", serde_json::to_string(r)?.as_str());
            }
            None => {}
        }
        let re = self
            .client
            .post_multipart(
                format!("{}/bot{}/sendAnimation", self.cfg.base, self.cfg.token),
                None,
                form,
            )
            .await
            .ok_or("Failed to send animation.")?;
        let status = re.status();
        match re.text().await {
            Ok(t) => Ok(serde_json::from_str(t.as_str())?),
            Err(e) => Err(format!("HTTP ERROR {}: {}", status, e))?,
        }
    }

    pub async fn send_document(
        &self,
        chat_id: &ChatId,
        message_thread_id: Option<i64>,
        document: InputFile,
        thumbnail: Option<InputFile>,
        caption: Option<&str>,
        parse_mode: Option<ParseMode>,
        disable_content_type_detection: Option<bool>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        message_effect_id: Option<&str>,
        reply_parameters: Option<&ReplyParameters>,
    ) -> Result<BotApiResult<Message>, BotapiClientError> {
        let mut form = FormData::new();
        form.data("chat_id", &chat_id.to_string());
        match message_thread_id {
            Some(m) => {
                form.data("message_thread_id", &m.to_string());
            }
            None => {}
        }
        match document {
            InputFile::URL(u) => {
                form.data("document", &u);
            }
            InputFile::Content(c) => {
                form.part("document", c);
            }
        }
        match thumbnail {
            Some(m) => match m {
                InputFile::URL(u) => {
                    form.data("thumbnail", &u);
                }
                InputFile::Content(c) => {
                    form.part("thumbnail", c);
                }
            },
            None => {}
        }
        match caption {
            Some(c) => {
                form.data("caption", c);
            }
            None => {}
        }
        match parse_mode {
            Some(p) => {
                form.data("parse_mode", p.as_ref());
            }
            None => {}
        }
        match disable_content_type_detection {
            Some(d) => {
                form.data("disable_content_type_detection", &d.to_string());
            }
            None => {}
        }
        match disable_notification {
            Some(d) => {
                form.data("disable_notification", &d.to_string());
            }
            None => {}
        }
        match protect_content {
            Some(p) => {
                form.data("protect_content", &p.to_string());
            }
            None => {}
        }
        match message_effect_id {
            Some(m) => {
                form.data("message_effect_id", m);
            }
            None => {}
        }
        match reply_parameters {
            Some(r) => {
                form.data("reply_parameters", serde_json::to_string(r)?.as_str());
            }
            None => {}
        }
        let re = self
            .client
            .post_multipart(
                format!("{}/bot{}/sendDocument", self.cfg.base, self.cfg.token),
                None,
                form,
            )
            .await
            .ok_or("Failed to send document.")?;
        let status = re.status();
        match re.text().await {
            Ok(t) => Ok(serde_json::from_str(t.as_str())?),
            Err(e) => Err(format!("HTTP ERROR {}: {}", status, e))?,
        }
    }

    pub async fn send_photo(
        &self,
        chat_id: &ChatId,
        message_thread_id: Option<i64>,
        photo: InputFile,
        caption: Option<&str>,
        parse_mode: Option<ParseMode>,
        show_caption_above_media: Option<bool>,
        has_spoiler: Option<bool>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        message_effect_id: Option<&str>,
        reply_parameters: Option<&ReplyParameters>,
    ) -> Result<BotApiResult<Message>, BotapiClientError> {
        let mut form = FormData::new();
        form.data("chat_id", &chat_id.to_string());
        match message_thread_id {
            Some(m) => {
                form.data("message_thread_id", &m.to_string());
            }
            None => {}
        }
        match photo {
            InputFile::URL(u) => {
                form.data("photo", &u);
            }
            InputFile::Content(c) => {
                form.part("photo", c);
            }
        }
        match caption {
            Some(c) => {
                form.data("caption", c);
            }
            None => {}
        }
        match parse_mode {
            Some(p) => {
                form.data("parse_mode", p.as_ref());
            }
            None => {}
        }
        match show_caption_above_media {
            Some(p) => {
                form.data("show_caption_above_media", &p.to_string());
            }
            None => {}
        }
        match has_spoiler {
            Some(p) => {
                form.data("has_spoiler", &p.to_string());
            }
            None => {}
        }
        match disable_notification {
            Some(d) => {
                form.data("disable_notification", &d.to_string());
            }
            None => {}
        }
        match protect_content {
            Some(p) => {
                form.data("protect_content", &p.to_string());
            }
            None => {}
        }
        match message_effect_id {
            Some(m) => {
                form.data("message_effect_id", m);
            }
            None => {}
        }
        match reply_parameters {
            Some(r) => {
                form.data("reply_parameters", serde_json::to_string(r)?.as_str());
            }
            None => {}
        }
        let re = self
            .client
            .post_multipart(
                format!("{}/bot{}/sendPhoto", self.cfg.base, self.cfg.token),
                None,
                form,
            )
            .await
            .ok_or("Failed to send photo.")?;
        let status = re.status();
        match re.text().await {
            Ok(t) => Ok(serde_json::from_str(t.as_str())?),
            Err(e) => Err(format!("HTTP ERROR {}: {}", status, e))?,
        }
    }

    pub async fn send_message<T: AsRef<str> + ?Sized>(
        &self,
        chat_id: &ChatId,
        message_thread_id: Option<i64>,
        text: &T,
        parse_mode: Option<ParseMode>,
        link_preview_options: Option<&LinkPreviewOptions>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        message_effect_id: Option<&str>,
        reply_parameters: Option<&ReplyParameters>,
    ) -> Result<BotApiResult<Message>, BotapiClientError> {
        let mut params = HashMap::new();
        params.insert("chat_id", chat_id.to_string());
        match message_thread_id {
            Some(m) => {
                params.insert("message_thread_id", m.to_string());
            }
            None => {}
        }
        params.insert("text", text.as_ref().to_owned());
        match parse_mode {
            Some(m) => {
                params.insert("parse_mode", m.as_ref().to_owned());
            }
            None => {}
        }
        match link_preview_options {
            Some(m) => {
                params.insert("link_preview_options", serde_json::to_string(m)?);
            }
            None => {}
        }
        match disable_notification {
            Some(b) => {
                params.insert("disable_notification", b.to_string());
            }
            None => {}
        }
        match protect_content {
            Some(b) => {
                params.insert("protect_content", b.to_string());
            }
            None => {}
        }
        match message_effect_id {
            Some(b) => {
                params.insert("message_effect_id", b.to_owned());
            }
            None => {}
        }
        match reply_parameters {
            Some(b) => {
                params.insert("reply_parameters", serde_json::to_string(b)?);
            }
            None => {}
        }
        let re = self
            .client
            .post(
                format!("{}/bot{}/sendMessage", self.cfg.base, self.cfg.token),
                None,
                Some(params),
            )
            .await
            .ok_or("Failed to send message.")?;
        let status = re.status();
        match re.text().await {
            Ok(t) => Ok(serde_json::from_str(t.as_str())?),
            Err(e) => Err(format!("HTTP ERROR {}: {}", status, e))?,
        }
    }

    pub async fn send_video<T: AsRef<str> + ?Sized>(
        &self,
        chat_id: &ChatId,
        message_thread_id: Option<i64>,
        video: InputFile,
        duration: Option<i64>,
        width: Option<i64>,
        height: Option<i64>,
        thumbnail: Option<InputFile>,
        caption: Option<&str>,
        parse_mode: Option<ParseMode>,
        show_caption_above_media: Option<bool>,
        has_spoiler: Option<bool>,
        supports_streaming: Option<bool>,
        disable_notification: Option<bool>,
        protect_content: Option<bool>,
        message_effect_id: Option<&str>,
        reply_parameters: Option<&ReplyParameters>,
    ) -> Result<BotApiResult<Message>, BotapiClientError> {
        let mut form = FormData::new();
        form.data("chat_id", &chat_id.to_string());
        match message_thread_id {
            Some(m) => {
                form.data("message_thread_id", &m.to_string());
            }
            None => {}
        }
        match video {
            InputFile::URL(u) => {
                form.data("video", &u);
            }
            InputFile::Content(c) => {
                form.part("video", c);
            }
        }
        match duration {
            Some(d) => {
                form.data("duration", &d.to_string());
            }
            None => {}
        }
        match width {
            Some(d) => {
                form.data("width", &d.to_string());
            }
            None => {}
        }
        match height {
            Some(d) => {
                form.data("height", &d.to_string());
            }
            None => {}
        }
        match thumbnail {
            Some(m) => match m {
                InputFile::URL(u) => {
                    form.data("thumbnail", &u);
                }
                InputFile::Content(c) => {
                    form.part("thumbnail", c);
                }
            },
            None => {}
        }
        match caption {
            Some(c) => {
                form.data("caption", c);
            }
            None => {}
        }
        match parse_mode {
            Some(p) => {
                form.data("parse_mode", p.as_ref());
            }
            None => {}
        }
        match show_caption_above_media {
            Some(p) => {
                form.data("show_caption_above_media", &p.to_string());
            }
            None => {}
        }
        match has_spoiler {
            Some(p) => {
                form.data("has_spoiler", &p.to_string());
            }
            None => {}
        }
        match supports_streaming {
            Some(s) => {
                form.data("supports_streaming", &s.to_string());
            }
            None => {}
        }
        match disable_notification {
            Some(d) => {
                form.data("disable_notification", &d.to_string());
            }
            None => {}
        }
        match protect_content {
            Some(p) => {
                form.data("protect_content", &p.to_string());
            }
            None => {}
        }
        match message_effect_id {
            Some(m) => {
                form.data("message_effect_id", m);
            }
            None => {}
        }
        match reply_parameters {
            Some(r) => {
                form.data("reply_parameters", serde_json::to_string(r)?.as_str());
            }
            None => {}
        }
        let re = self
            .client
            .post_multipart(
                format!("{}/bot{}/sendVideo", self.cfg.base, self.cfg.token),
                None,
                form,
            )
            .await
            .ok_or("Failed to send video.")?;
        let status = re.status();
        match re.text().await {
            Ok(t) => Ok(serde_json::from_str(t.as_str())?),
            Err(e) => Err(format!("HTTP ERROR {}: {}", status, e))?,
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_telegram_botapi_sendmessage() {
    match std::env::var("TGBOT_TOKEN") {
        Ok(token) => match std::env::var("TGBOT_CHATID") {
            Ok(c) => {
                let cfg = BotapiClientConfigBuilder::default()
                    .token(token)
                    .build()
                    .unwrap();
                let client = BotapiClient::new(&cfg);
                let cid = ChatId::try_from(c).unwrap();
                let data = client
                    .send_message(
                        &cid,
                        None,
                        "Hello world.",
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap()
                    .unwrap();
                let r = ReplyParametersBuilder::default()
                    .message_id(data.message_id)
                    .build()
                    .unwrap();
                client
                    .send_message(
                        &cid,
                        data.message_thread_id,
                        "Reply message",
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(&r),
                    )
                    .await
                    .unwrap()
                    .unwrap();
            }
            Err(_) => {
                println!("No chat id specified, skip test.")
            }
        },
        Err(_) => {
            println!("No tg bot token specified, skip test.")
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_telegram_botapi_sendphoto() {
    match std::env::var("TGBOT_TOKEN") {
        Ok(token) => match std::env::var("TGBOT_CHATID") {
            Ok(c) => {
                let cfg = BotapiClientConfigBuilder::default()
                    .token(token)
                    .build()
                    .unwrap();
                let client = BotapiClient::new(&cfg);
                let cid = ChatId::try_from(c).unwrap();
                let pb = std::path::PathBuf::from("./testdata/夏のチマメ隊🏖️_91055644_p0.jpg");
                let c = FormDataPartBuilder::default()
                    .body(pb)
                    .filename("夏のチマメ隊🏖️_91055644_p0.jpg")
                    .mime("image/jpeg")
                    .build()
                    .unwrap();
                client
                    .send_photo(
                        &cid,
                        None,
                        InputFile::Content(c),
                        Some("test.test.test"),
                        None,
                        None,
                        Some(true),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap()
                    .unwrap();
            }
            Err(_) => {
                println!("No chat id specified, skip test.")
            }
        },
        Err(_) => {
            println!("No tg bot token specified, skip test.")
        }
    }
}
