use crate::ext::json::FromJson;
use crate::ext::use_or_not::UseOrNot;
use crate::gettext;
use crate::retry_interval::check_retry_interval;
use crate::settings::SettingDes;
use crate::settings::JsonValueType;
use crate::opt::author_name_filter::check_author_name_filters;
use crate::opt::proxy::check_proxy;
use crate::opt::size::parse_u32_size;
#[cfg(feature = "server")]
use crate::server::cors::parse_cors_entries;
use json::JsonValue;
#[cfg(feature = "server")]
use std::net::SocketAddr;
#[cfg(feature = "server")]
use std::str::FromStr;

pub fn get_settings_list() -> Vec<SettingDes> {
    vec![
        SettingDes::new("refresh_tokens", gettext("Pixiv's refresh tokens. Used to login."), JsonValueType::Str, None).unwrap(),
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
        SettingDes::new("max-threads", gettext("The maximun threads when downloading file."), JsonValueType::Number, Some(check_u64)).unwrap(),
        SettingDes::new("part-size", gettext("The size of the each part when downloading file."), JsonValueType::Number, Some(check_parse_size_u32)).unwrap(),
        SettingDes::new("proxy", gettext("Proxy settings."), JsonValueType::Array, Some(check_proxy)).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("server", gettext("Server address."), JsonValueType::Str, Some(check_socket_addr)).unwrap(),
        #[cfg(feature = "server")]
        SettingDes::new("cors-entries", gettext("The domains allowed to send CORS requests."), JsonValueType::Array, Some(check_cors_entries)).unwrap(),
        SettingDes::new("max-download-tasks", gettext("The maximun number of tasks to download simultaneously."), JsonValueType::Number, Some(check_nozero_usize)).unwrap(),
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
            println!("{}", e);
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

#[test]
fn test_get_settings_list() {
    get_settings_list();
}
