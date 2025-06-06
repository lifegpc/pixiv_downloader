use crate::downloader::DownloaderError;
use crate::ugoira::UgoiraError;
use tokio::task::JoinError;

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum PixivDownloaderError {
    DownloaderError(DownloaderError),
    String(String),
    JoinError(JoinError),
    UgoiraError(UgoiraError),
    #[cfg(feature = "server")]
    Hyper(hyper::Error),
    HTTP(http::Error),
    IOError(std::io::Error),
    Fanbox(crate::fanbox::error::FanboxAPIError),
    #[cfg(feature = "avdict")]
    AVDict(crate::avdict::AVDictError),
    #[cfg(feature = "db")]
    DbError(crate::db::PixivDownloaderDbError),
    #[cfg(feature = "server")]
    FromUtf8Error(std::string::FromUtf8Error),
    #[cfg(feature = "server")]
    ToStrError(http::header::ToStrError),
    JSONError(json::Error),
    ParseIntError(std::num::ParseIntError),
    ReqwestError(wreq::Error),
    PixivAppError(crate::pixivapp::error::PixivAppError),
    SerdeJsonError(serde_json::Error),
    #[cfg(feature = "serde_urlencoded")]
    SerdeUrlencodedError(serde_urlencoded::ser::Error),
    BotApiError(crate::push::telegram::botapi_client::BotapiClientError),
}

impl std::error::Error for PixivDownloaderError {}

impl From<&str> for PixivDownloaderError {
    fn from(p: &str) -> Self {
        Self::String(String::from(p))
    }
}

impl From<http::header::InvalidHeaderValue> for PixivDownloaderError {
    fn from(v: http::header::InvalidHeaderValue) -> Self {
        Self::HTTP(http::Error::from(v))
    }
}

impl PixivDownloaderError {
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::PixivAppError(e) => e.is_not_found(),
            _ => false,
        }
    }
}

#[macro_export]
macro_rules! concat_pixiv_downloader_error {
    ($exp1:expr, $exp2:expr) => {
        $exp1 = match $exp1 {
            Ok(x) => match $exp2 {
                Ok(_) => Ok(x),
                Err(e) => Err(PixivDownloaderError::from(e)),
            },
            Err(e) => match $exp2 {
                Ok(_) => Err(e),
                Err(e2) => {
                    log::error!("{}", e);
                    Err(PixivDownloaderError::from(e2))
                }
            },
        }
    };
}

#[macro_export]
macro_rules! concat_error {
    ($exp1:expr, $exp2:expr, $typ:ty) => {
        $exp1 = match $exp1 {
            Ok(x) => match $exp2 {
                Ok(_) => Ok(x),
                Err(e) => Err(<$typ>::from(e)),
            },
            Err(e) => match $exp2 {
                Ok(_) => Err(e),
                Err(e2) => {
                    log::error!("{}", e);
                    Err(<$typ>::from(e2))
                }
            },
        }
    };
}
