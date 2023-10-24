use crate::webclient::WebClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Text type
pub enum EveryPushTextType {
    Text,
    Image,
    Markdown,
}

impl AsRef<str> for EveryPushTextType {
    fn as_ref(&self) -> &str {
        match self {
            EveryPushTextType::Text => "text",
            EveryPushTextType::Image => "image",
            EveryPushTextType::Markdown => "markdown",
        }
    }
}

impl std::fmt::Display for EveryPushTextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", AsRef::<str>::as_ref(self))
    }
}

pub struct EveryPushClient {
    client: WebClient,
    server: String,
}

impl EveryPushClient {
    pub fn new(server: String) -> Self {
        Self {
            client: WebClient::default(),
            server,
        }
    }

    /// push message to server
    /// * `push_token` - push token
    /// * `text` - text
    /// * `title` - title
    /// * `topic_id` - topic id
    /// * `typ` - text type
    ///
    /// For more information, see [API document](https://github.com/PeanutMelonSeedBigAlmond/EveryPush.Server/blob/main/api.md#推送消息)
    pub async fn push_message<
        P: AsRef<str> + ?Sized,
        T: AsRef<str> + ?Sized,
        I: AsRef<str> + ?Sized,
        O: AsRef<str> + ?Sized,
    >(
        &self,
        push_token: &P,
        text: &T,
        title: Option<&I>,
        topic_id: Option<&O>,
        typ: Option<EveryPushTextType>,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("pushToken", push_token.as_ref());
        params.insert("text", text.as_ref());
        match title {
            Some(t) => params.insert("title", t.as_ref()),
            None => None,
        };
        match topic_id {
            Some(t) => params.insert("topicId", t.as_ref()),
            None => None,
        };
        match &typ {
            Some(t) => params.insert("type", t.as_ref()),
            None => None,
        };
        let re = self
            .client
            .post(format!("{}/message/push", self.server), None, Some(params))
            .await
            .ok_or("Failed to send push message.")?;
        let status = re.status();
        if status.is_success() {
            Ok(())
        } else {
            match re.text().await {
                Ok(t) => match json::parse(t.as_str()) {
                    Ok(v) => {
                        let msg = v["message"].as_str();
                        match msg {
                            Some(m) => Err(m.to_owned()),
                            None => Err(format!("HTTP ERROR {}", status)),
                        }
                    }
                    Err(e) => Err(format!("HTTP ERROR {}: {}", status, e)),
                },
                Err(e) => Err(format!("HTTP ERROR {}: {}", status, e)),
            }
        }
    }
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_every_push_push() {
    match std::env::var("EVERY_PUSH_SERVER") {
        Ok(server) => match std::env::var("EVERY_PUSH_TOKEN") {
            Ok(token) => {
                let client = EveryPushClient::new(server);
                match client
                    .push_message(
                        &token,
                        "Push Test",
                        Some("Push"),
                        Some("test"),
                        Some(EveryPushTextType::Text),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
            Err(_) => {
                println!("No every push token specified, skip test.")
            }
        },
        Err(_) => {
            println!("No every push server specified, skip test.")
        }
    }
}
