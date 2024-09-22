#[cfg(feature = "db")]
use crate::db::check_db_config;
use crate::ext::json::FromJson;
use crate::ext::use_or_not::UseOrNot;
use crate::gettext;
use crate::retry_interval::check_retry_interval;
use crate::settings::SettingDes;
use crate::settings::JsonValueType;
use crate::opt::author_name_filter::check_author_name_filters;
use crate::opt::crf::check_crf;
use crate::opt::header_map::check_header_map;
use crate::opt::proxy::check_proxy;
use crate::opt::size::parse_u32_size;
#[cfg(feature = "server")]
use crate::server::cors::parse_cors_entries;
use crate::ugoira::X264Profile;
use json::JsonValue;
#[cfg(feature = "server")]
use std::net::SocketAddr;
use std::str::FromStr;

pub fn get_settings_list() -> Vec<SettingDes> {
    vec![
        SettingDes::new("refresh-token", gettext("Pixiv's refresh token. Used to login."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("cookies", gettext("The location of cookies file. Used for web API."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("language", gettext("The language of translated tags."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("retry", gettext("Max retry count if request failed."), JsonValueType::Number, Some(check_i64)).unwrap(),
        SettingDes::new("retry-interval", gettext("The interval (in seconds) between two retries."), JsonValueType::Multiple, Some(check_retry_interval)).unwrap(),
        SettingDes::new("use-webpage", gettext("Use data from webpage first."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("author-name-filters", gettext("Remove the part which after these parttens."), JsonValueType::Array, Some(check_author_name_filters)).unwrap(),
        #[cfg(feature = "exif")]
        SettingDes::new("update-exif", gettext("Add/Update exif information to image files even when overwrite are disabled."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("progress-bar-template", gettext("Progress bar's template. See <here> for more informations.").replace("<here>", "https://docs.rs/indicatif/latest/indicatif/#templates").as_str(), JsonValueType::Str, Some(check_nonempty_str)).unwrap(),
        SettingDes::new("use-progress-bar", gettext("Whether to enable progress bar."), JsonValueType::Multiple, Some(check_user_or_not)).unwrap(),
        SettingDes::new("download-multiple-files", gettext("Download multiple files at the same time."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("download-retry", gettext("Max retry count if download failed."), JsonValueType::Number, Some(check_i64)).unwrap(),
        SettingDes::new("download-retry-interval", gettext("The interval (in seconds) between two retries when downloading files."), JsonValueType::Multiple, Some(check_retry_interval)).unwrap(),
        SettingDes::new("multiple-threads-download", gettext("Whether to enable multiple threads download."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("download-part-retry", gettext("Max retry count of each part when downloading in multiple thread mode."), JsonValueType::Number, Some(check_i64)).unwrap(),
        SettingDes::new("max-threads", gettext("The maximum threads when downloading file."), JsonValueType::Number, Some(check_u64)).unwrap(),
        SettingDes::new("part-size", gettext("The size of the each part when downloading file."), JsonValueType::Number, Some(check_parse_size_u32)).unwrap(),
        SettingDes::new("proxy", gettext("Proxy settings."), JsonValueType::Array, Some(check_proxy)).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("server", gettext("Server address."), JsonValueType::Str, Some(check_socket_addr)).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("cors-entries", gettext("The domains allowed to send CORS requests."), JsonValueType::Array, Some(check_cors_entries)).unwrap(),
        SettingDes::new("max-download-tasks", gettext("The maximum number of tasks to download files at the same time."), JsonValueType::Number, Some(check_nozero_usize)).unwrap(),
        SettingDes::new("download-multiple-posts", gettext("Download multiple posts/artworks at the same time."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("max-download-post-tasks", gettext("The maximum number of tasks to download posts/artworks at the same time."), JsonValueType::Number, Some(check_nozero_usize)).unwrap(),
        SettingDes::new("force-yuv420p", gettext("Force yuv420p as output pixel format when converting ugoira(GIF) to video."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("x264-profile", gettext("The x264 profile when converting ugoira(GIF) to video."), JsonValueType::Str, Some(check_x264_profile)).unwrap(),
        #[cfg(feature = "db")]
        SettingDes::new("db", gettext("Database settings."), JsonValueType::Object, Some(check_db_config)).unwrap(),
        SettingDes::new("download-base", gettext("The base directory to save downloaded files."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("user-agent", gettext("The User-Agent header."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("x264-crf", gettext("The Constant Rate Factor when converting ugoira(GIF) to video."), JsonValueType::Number, Some(check_crf)).unwrap(),
        SettingDes::new("ugoira-max-fps", gettext("The max fps when converting ugoira(GIF) to video."), JsonValueType::Number, Some(check_ugoira_max_fps)).unwrap(),
        SettingDes::new("fanbox-page-number", gettext("Use page number for pictures' file name in fanbox."), JsonValueType::Boolean, None).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("cors-allow-all", gettext("Whether to allow all domains to send CORS requests."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("use-app-api", gettext("Whether to use Pixiv APP API first."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("use-web-description", gettext("Whether to use description from Web API when description from APP API is empty."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("add-history", gettext("Whether to add artworks to pixiv's history. Only works for APP API."), JsonValueType::Boolean, None).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("server-base", gettext("The server's host name. Used in some proxy."), JsonValueType::Str, None).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("push-task-max-count", gettext("The maximum number of push tasks running at the same time."), JsonValueType::Number, Some(check_nozero_usize)).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("push-task-max-push-count", gettext("The maximum number of tasks to push to client at the same time."), JsonValueType::Number, Some(check_nozero_usize)).unwrap(),
        SettingDes::new("fanbox-http-headers", gettext("Extra http headers for fanbox.cc."), JsonValueType::Object, Some(check_header_map)).unwrap(),
        SettingDes::new("log-cfg", gettext("The path to the config file of log4rs."), JsonValueType::Str, None).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("ffprobe", gettext("The path to ffprobe executable."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("ugoira", gettext("The path to ugoira cli executable."), JsonValueType::Str, None).unwrap(),
        #[cfg(feature = "ugoira")]
        SettingDes::new("ugoira-cli", gettext("Whether to use ugoira cli."), JsonValueType::Boolean, None).unwrap(),
    ]
}

fn check_i64(obj: &JsonValue) -> bool {
    let r = obj.as_i64();
    r.is_some()
}

#[cfg(feature = "server")]
fn check_cors_entries(obj: &JsonValue) -> bool {
    match parse_cors_entries(obj) {
        Ok(_) => true,
        Err(e) => {
            log::error!("{}", e);
            false
        }
    }
}

#[cfg(feature = "server")]
fn check_socket_addr(obj: &JsonValue) -> bool {
    match obj.as_str() {
        Some(s) => match SocketAddr::from_str(s) {
            Ok(_) => true,
            Err(_) => false,
        }
        None => false,
    }
}

fn check_nozero_usize(obj: &JsonValue) -> bool {
    let r = obj.as_usize();
    r.is_some() && r.unwrap() > 0
}

fn check_u64(obj: &JsonValue) -> bool {
    let r = obj.as_u64();
    r.is_some()
}

#[inline]
fn check_parse_size_u32(obj: &JsonValue) -> bool {
    parse_u32_size(obj).is_some()
}

fn check_nonempty_str(obj: &JsonValue) -> bool {
    let r = obj.as_str();
    r.is_some() && r.unwrap().len() != 0
}

fn check_user_or_not(obj: &JsonValue) -> bool {
    let r = UseOrNot::from_json(obj);
    r.is_ok()
}

fn check_x264_profile(obj: &JsonValue) -> bool {
    match obj.as_str() {
        Some(profile) => X264Profile::from_str(profile).is_ok(),
        None => false,
    }
}

fn check_ugoira_max_fps(obj: &JsonValue) -> bool {
    match obj.as_f32() {
        Some(fps) => fps > 0f32 && fps <= 1000f32,
        None => false,
    }
}

#[test]
fn test_get_settings_list() {
    get_settings_list();
}
