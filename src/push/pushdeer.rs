use crate::ext::try_err::TryErr4;
use crate::webclient::WebClient;
use proc_macros::pushdeer_api_quick_test;
use std::collections::HashMap;

pub struct PushdeerClient {
    client: WebClient,
    server: String,
}

impl PushdeerClient {
    pub fn new(server: String) -> Self {
        Self {
            client: WebClient::default(),
            server,
        }
    }

    async fn handle_result(re: reqwest::Response) -> Result<(), String> {
        let obj = json::parse(
            re.text()
                .await
                .try_err4("Failed to read text from response: ")?
                .as_str(),
        )
        .try_err4("Failed to parse JSON: ")?;
        let code = obj["code"].as_i64().ok_or("Failed to get code.")?;
        if code == 0 {
            Ok(())
        } else {
            let msg = obj["error"]
                .as_str()
                .ok_or("Failed to get error message.")?;
            Err(msg.to_owned())
        }
    }

    /// push text message to server
    /// * `pushkey` - push key
    /// * `text` - text
    pub async fn push_text_message(&self, pushkey: String, text: String) -> Result<(), String> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(String::from("pushkey"), pushkey);
        params.insert(String::from("text"), text);
        let re = self
            .client
            .post(format!("{}/message/push", self.server), None, Some(params))
            .await
            .ok_or("Failed to send text message.")?;
        Self::handle_result(re).await
    }

    /// push image message to server
    /// * `pushkey` - push key
    /// * `image` - image URL
    pub async fn push_image(&self, pushkey: String, image: String) -> Result<(), String> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(String::from("pushkey"), pushkey);
        params.insert(String::from("text"), image);
        params.insert(String::from("type"), String::from("image"));
        let re = self
            .client
            .post(format!("{}/message/push", self.server), None, Some(params))
            .await
            .ok_or("Failed to send image message.")?;
        Self::handle_result(re).await
    }

    /// push markdown message to server
    /// * `pushkey` - push key
    /// * `title` - title
    /// * `text` - markdown text
    pub async fn push_markdown_message(
        &self,
        pushkey: String,
        title: String,
        text: String,
    ) -> Result<(), String> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert(String::from("pushkey"), pushkey);
        params.insert(String::from("text"), title);
        params.insert(String::from("desp"), text);
        params.insert(String::from("type"), String::from("markdown"));
        let re = self
            .client
            .post(format!("{}/message/push", self.server), None, Some(params))
            .await
            .ok_or("Failed to send markdown message.")?;
        Self::handle_result(re).await
    }
}

pushdeer_api_quick_test!(
    test_push_text_message,
    client.push_text_message(token, String::from("Test message")),
    "Failed to send text message:"
);

pushdeer_api_quick_test!(
    test_push_image,
    client.push_image(token, String::from("https://pd.lifegpc.com/proxy/pixiv/112199539_p0.jpg?url=https%3A%2F%2Fi.pximg.net%2Fimg-original%2Fimg%2F2023%2F10%2F02%2F00%2F00%2F38%2F112199539_p0.jpg&sign=4f429e69218669e226e1171d2499f49ac7f41f56c3a8275619641c0a3327a394dae63dcd145367d66a3735279dc59b39219eb4d4c20bfe5afb7e801a2e7c2496")),
    "Failed to send image message:"
);

pushdeer_api_quick_test!(
    test_push_markdown_message,
    client.push_markdown_message(
        token,
        String::from("Test title"),
        String::from("# Test Header\n[link](https://github.com/lifegpc/pixiv_downloader)")
    ),
    "Failed to send markdown message:"
);
