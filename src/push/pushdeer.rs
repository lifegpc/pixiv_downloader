use crate::ext::try_err::TryErr4;
use crate::webclient::WebClient;
use proc_macros::pushdeer_api_quick_test;
use std::collections::HashMap;

pub struct PushdeerClient {
    client: WebClient,
    server: String,
}

impl PushdeerClient {
    pub fn new<S: AsRef<str> + ?Sized>(server: &S) -> Self {
        Self {
            client: WebClient::default(),
            server: server.as_ref().to_owned(),
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
    pub async fn push_text_message<P: AsRef<str> + ?Sized, T: AsRef<str> + ?Sized>(
        &self,
        pushkey: &P,
        text: &T,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("pushkey", pushkey.as_ref());
        params.insert("text", text.as_ref());
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
    pub async fn push_image<P: AsRef<str> + ?Sized, I: AsRef<str> + ?Sized>(
        &self,
        pushkey: &P,
        image: &I,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("pushkey", pushkey.as_ref());
        params.insert("text", image.as_ref());
        params.insert("type", "image");
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
    pub async fn push_markdown_message<
        P: AsRef<str> + ?Sized,
        T: AsRef<str> + ?Sized,
        E: AsRef<str> + ?Sized,
    >(
        &self,
        pushkey: &P,
        title: &T,
        text: &E,
    ) -> Result<(), String> {
        let mut params = HashMap::new();
        params.insert("pushkey", pushkey.as_ref());
        params.insert("text", title.as_ref());
        params.insert("desp", text.as_ref());
        params.insert("type", "markdown");
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
    client.push_text_message(&token, "Test message"),
    "Failed to send text message:"
);

pushdeer_api_quick_test!(
    test_push_image,
    client.push_image(&token, "https://pd.lifegpc.com/proxy/pixiv/112199539_p0.jpg?url=https%3A%2F%2Fi.pximg.net%2Fimg-original%2Fimg%2F2023%2F10%2F02%2F00%2F00%2F38%2F112199539_p0.jpg&sign=4f429e69218669e226e1171d2499f49ac7f41f56c3a8275619641c0a3327a394dae63dcd145367d66a3735279dc59b39219eb4d4c20bfe5afb7e801a2e7c2496"),
    "Failed to send image message:"
);

pushdeer_api_quick_test!(
    test_push_markdown_message,
    client.push_markdown_message(
        &token,
        "Test title",
        "# Test Header\n[link](https://github.com/lifegpc/pixiv_downloader)"
    ),
    "Failed to send markdown message:"
);
