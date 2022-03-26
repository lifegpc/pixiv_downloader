use crate::author_name_filter::check_author_name_filters;
use crate::ext::json::FromJson;
use crate::ext::use_or_not::UseOrNot;
use crate::gettext;
use crate::retry_interval::check_retry_interval;
use crate::settings::SettingDes;
use crate::settings::JsonValueType;
use json::JsonValue;

pub fn get_settings_list() -> Vec<SettingDes> {
    vec![
        SettingDes::new("refresh_tokens", gettext("Pixiv's refresh tokens. Used to login."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("cookies", gettext("The location of cookies file. Used for web API."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("language", gettext("The language of translated tags."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("retry", gettext("Max retry count if request failed."), JsonValueType::Number, Some(check_u64)).unwrap(),
        SettingDes::new("retry-interval", gettext("The interval (in seconds) between two retries."), JsonValueType::Multiple, Some(check_retry_interval)).unwrap(),
        SettingDes::new("use-webpage", gettext("Use data from webpage first."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("author-name-filters", gettext("Remove the part which after these parttens."), JsonValueType::Array, Some(check_author_name_filters)).unwrap(),
        #[cfg(feature = "exif")]
        SettingDes::new("update-exif", gettext("Add/Update exif information to image files even when overwrite are disabled."), JsonValueType::Boolean, None).unwrap(),
        SettingDes::new("progress-bar-template", gettext("Progress bar's template. See <here> for more informations.").replace("<here>", "https://docs.rs/indicatif/latest/indicatif/#templates").as_str(), JsonValueType::Str, Some(check_nonempty_str)).unwrap(),
        SettingDes::new("use-progress-bar", gettext("Whether to enable progress bar."), JsonValueType::Multiple, Some(check_user_or_not)).unwrap(),
    ]
}

fn check_u64(obj: &JsonValue) -> bool {
    let r = obj.as_u64();
    r.is_some()
}

fn check_nonempty_str(obj: &JsonValue) -> bool {
    let r = obj.as_str();
    r.is_some() && r.unwrap().len() != 0
}

fn check_user_or_not(obj: &JsonValue) -> bool {
    let r = UseOrNot::from_json(obj);
    r.is_ok()
}
