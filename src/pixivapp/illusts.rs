use super::illust::PixivAppIllust;
use crate::error::PixivDownloaderError;
use crate::gettext;
use crate::pixiv_app::PixivAppClientInternal;
use json::JsonValue;
use std::sync::Arc;

pub struct PixivAppIllusts {
    pub illusts: Vec<PixivAppIllust>,
    next_url: Option<String>,
    client: Arc<PixivAppClientInternal>,
}

impl PixivAppIllusts {
    #[allow(dead_code)]
    /// Get next page.
    /// # Note
    /// If no next page presented, will return a error.
    pub async fn get_next_page(&self) -> Result<Self, PixivDownloaderError> {
        match &self.next_url {
            Some(url) => Self::new(
                self.client.clone(),
                self.client
                    .get_url(
                        url,
                        gettext("Failed to get next page of the illusts."),
                        gettext("Illusts data:"),
                    )
                    .await?,
            ),
            None => Err("No next url, can not get next page.".into()),
        }
    }
    #[allow(dead_code)]
    /// Returns true if next page is presented.
    pub fn has_next_page(&self) -> bool {
        self.next_url.is_some()
    }

    pub fn new(
        client: Arc<PixivAppClientInternal>,
        value: JsonValue,
    ) -> Result<Self, PixivDownloaderError> {
        let oillusts = &value["illusts"];
        if !oillusts.is_array() {
            return Err(gettext("Failed to parse illust list.").into());
        }
        let mut illusts = Vec::new();
        for item in oillusts.members() {
            illusts.push(PixivAppIllust::new(item.clone()));
        }
        let next_url = match value["nextUrl"].as_str() {
            Some(next_url) => Some(next_url.to_owned()),
            None => None,
        };
        Ok(Self {
            illusts,
            next_url,
            client,
        })
    }
}
