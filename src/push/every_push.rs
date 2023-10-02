use crate::webclient::WebClient;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Text type
pub enum EveryPushTextType {
    Text,
    Image,
    Markdown,
}

impl std::fmt::Display for EveryPushTextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EveryPushTextType::Text => write!(f, "text"),
            EveryPushTextType::Image => write!(f, "image"),
            EveryPushTextType::Markdown => write!(f, "markdown"),
        }
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
    /// * `typ` - text type
    ///
    /// For more information, see [API document](https://github.com/PeanutMelonSeedBigAlmond/EveryPush.Server/blob/main/api.md#推送消息)
    pub async fn push_message(
        &self,
        push_token: String,
        text: String,
        title: Option<String>,
        typ: Option<EveryPushTextType>,
    ) -> Result<(), String> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(String::from("pushToken"), push_token);
        params.insert(String::from("text"), text);
        match title {
            Some(t) => params.insert(String::from("title"), t),
            None => None,
        };
        match typ {
            Some(t) => params.insert(String::from("type"), format!("{}", t)),
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
                        token,
                        String::from("Push Test"),
                        Some(String::from("Push")),
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
