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
}

impl From<&str> for PixivDownloaderError {
    fn from(p: &str) -> Self {
        Self::String(String::from(p))
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