use crate::downloader::DownloaderError;
#[cfg(feature = "ugoira")]
use crate::ugoira::UgoiraError;
use tokio::task::JoinError;

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum PixivDownloaderError {
    DownloaderError(DownloaderError),
    String(String),
    JoinError(JoinError),
    #[cfg(feature = "ugoira")]
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
    #[cfg(all(feature = "server", feature = "db_sqlite", test))]
    JSONError(json::Error),
    #[cfg(feature = "openssl")]
    OpenSSLError(openssl::error::ErrorStack),
    ParseIntError(std::num::ParseIntError),
    ReqwestError(reqwest::Error),
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
                    println!("{}", e);
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
                    println!("{}", e);
                    Err(<$typ>::from(e2))
                }
            },
        }
    };
}
