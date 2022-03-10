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
    ]
}

fn check_u64(obj: &JsonValue) -> bool {
    let r = obj.as_u64();
    r.is_some()
}
