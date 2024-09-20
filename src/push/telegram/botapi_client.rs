use super::tg_type::*;
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
