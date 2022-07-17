use super::downloader::Downloader;
use super::enums::DownloaderResult;
use super::error::DownloaderError;
use super::local_file::LocalFile;
use crate::ext::replace::ReplaceWith;
use crate::ext::try_err::TryErr;
use crate::gettext;
use crate::webclient::ToHeaders;
use crate::webclient::WebClient;
use reqwest::IntoUrl;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// Store download information
pub struct DownloaderHelper {
    /// Url
    pub url: Url,
    /// Web client
    pub client: Option<Arc<WebClient>>,
    /// New headers wants to apply
    pub headers: Option<HashMap<String, String>>,
    /// Recommand file name
    pub file_name: Option<PathBuf>,
}

pub struct DownloaderHelperBuilder {
    helper: DownloaderHelper,
}

impl DownloaderHelper {
    pub fn builder<U: IntoUrl>(url: U) -> Result<DownloaderHelperBuilder, DownloaderError> {
        Ok(DownloaderHelperBuilder {
            helper: Self {
                url: url.into_url()?,
                client: None,
                headers: None,
                file_name: None,
            },
        })
    }

    /// Get A local [Downloader]
    /// * `overwrite` - Whether to overwrite file
    /// * `base` - The base directory to store downloaded files.
    pub fn download_local<P: AsRef<Path> + ?Sized>(
        &self,
        overwrite: Option<bool>,
        base: &P,
    ) -> Result<DownloaderResult<Downloader<LocalFile>>, DownloaderError> {
        let file = self
            .get_local_file_path(base)
            .try_err(gettext("Failed to get file name from url."))?;
        match &self.client {
            Some(client) => Downloader::<LocalFile>::new2(
                Arc::clone(client),
                self.url.clone(),
                self.headers.clone(),
                Some(&file),
                overwrite,
            ),
            None => Downloader::<LocalFile>::new(
                self.url.clone(),
                self.headers.clone(),
                Some(&file),
                overwrite,
            ),
        }
    }

    pub fn get_local_file_path<P: AsRef<Path> + ?Sized>(
        &self,
        base: &P,
    ) -> Option<std::path::PathBuf> {
        let base = base.as_ref();
        match &self.file_name {
            Some(file_name) => Some(base.join(file_name)),
            None => match crate::utils::get_file_name_from_url(self.url.clone()) {
                Some(file_name) => Some(base.join(file_name)),
                None => None,
            },
        }
    }

    pub fn set_client(&mut self, client: &Arc<WebClient>) {
        self.client.replace(Arc::clone(client));
    }

    pub fn set_file_name<P: AsRef<Path> + ?Sized>(&mut self, p: &P) {
        self.file_name.replace(p.as_ref().to_owned());
    }

    pub fn set_headers<H: ToHeaders>(&mut self, headers: H) {
        self.headers.replace_with(match headers.to_headers() {
            Some(headers) => Some(headers),
            None => None,
        });
    }

    pub fn set_no_client(&mut self) {
        self.client.take();
    }

    pub fn set_no_file_name(&mut self) {
        self.file_name.take();
    }

    pub fn set_no_headers(&mut self) {
        self.headers.take();
    }
}

impl DownloaderHelperBuilder {
    pub fn build(self) -> DownloaderHelper {
        self.helper
    }

    pub fn client(mut self, client: &Arc<WebClient>) -> Self {
        self.helper.set_client(client);
        self
    }

    pub fn file_name<P: AsRef<Path> + ?Sized>(mut self, p: &P) -> Self {
        self.helper.set_file_name(p);
        self
    }

    pub fn headers<H: ToHeaders>(mut self, headers: H) -> Self {
        self.helper.set_headers(headers);
        self
    }

    pub fn no_client(mut self) -> Self {
        self.helper.set_no_client();
        self
    }

    pub fn no_file_name(mut self) -> Self {
        self.helper.set_no_file_name();
        self
    }

    pub fn no_headers(mut self) -> Self {
        self.helper.set_no_headers();
        self
    }
}

impl From<Url> for DownloaderHelper {
    fn from(url: Url) -> Self {
        Self {
            url,
            client: None,
            headers: None,
            file_name: None,
        }
    }
}

impl From<&Url> for DownloaderHelper {
    fn from(u: &Url) -> Self {
        Self::from(u.clone())
    }
}

impl<'a> TryFrom<&'a str> for DownloaderHelper {
    type Error = url::ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self::from(Url::parse(value)?))
    }
}
