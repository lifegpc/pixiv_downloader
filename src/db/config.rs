use crate::ext::try_err::TryErr;
use crate::gettext;
use json::JsonValue;

#[cfg(feature = "db_sqlite")]
pub struct PixivDownloaderSqliteConfig {
    /// The path of database file
    pub path: String,
}

#[cfg(feature = "db_sqlite")]
impl Default for PixivDownloaderSqliteConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "docker")]
            path: "/app/data/data.db".to_string(),
            #[cfg(not(feature = "docker"))]
            path: "pixiv_downloader.db".to_string(),
        }
    }
}

pub enum PixivDownloaderDbConfig {
    #[cfg(feature = "db_sqlite")]
    Sqlite(PixivDownloaderSqliteConfig),
    #[allow(dead_code)]
    /// No default config is provided
    None,
}

#[derive(Debug)]
pub enum PixivDownloaderDbConfigError {
    UnkonwnDbType,
    MissingField(String),
}

impl std::fmt::Display for PixivDownloaderDbConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnkonwnDbType => write!(f, "{}", gettext("Unknown database type.")),
            Self::MissingField(s) => write!(f, "{} {}", gettext("Missing field:"), s),
        }
    }
}

impl PixivDownloaderDbConfig {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn new(value: &JsonValue) -> Result<Self, PixivDownloaderDbConfigError> {
        let db_type = value["type"]
            .as_str()
            .try_err(PixivDownloaderDbConfigError::UnkonwnDbType)?;
        match db_type {
            #[cfg(feature = "db_sqlite")]
            "sqlite" => {
                let path =
                    value["path"]
                        .as_str()
                        .try_err(PixivDownloaderDbConfigError::MissingField(
                            "path".to_string(),
                        ))?;
                Ok(Self::Sqlite(PixivDownloaderSqliteConfig {
                    path: path.to_string(),
                }))
            }
            _ => Err(PixivDownloaderDbConfigError::UnkonwnDbType),
        }
    }
}

impl AsRef<PixivDownloaderDbConfig> for PixivDownloaderDbConfig {
    fn as_ref(&self) -> &PixivDownloaderDbConfig {
        self
    }
}

impl Default for PixivDownloaderDbConfig {
    fn default() -> Self {
        #[cfg(feature = "db_sqlite")]
        return Self::Sqlite(PixivDownloaderSqliteConfig::default());
        #[cfg(not(feature = "db_sqlite"))]
        return Self::None;
    }
}

pub fn check_db_config(value: &JsonValue) -> bool {
    PixivDownloaderDbConfig::new(value).is_ok()
}
