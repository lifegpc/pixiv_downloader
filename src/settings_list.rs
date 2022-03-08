use crate::gettext;
use crate::settings::SettingDes;
use crate::settings::JsonValueType;
use json::JsonValue;

pub fn get_settings_list() -> Vec<SettingDes> {
    vec![
        SettingDes::new("refresh_tokens", gettext("Pixiv's refresh tokens. Used to login."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("cookies", gettext("The location of cookies file. Used for web API."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("language", gettext("The language of translated tags."), JsonValueType::Str, None).unwrap(),
        SettingDes::new("retry", gettext("Max retry count if request failed."), JsonValueType::Number, Some(check_u64)).unwrap(),
    ]
}

fn check_u64(obj: &JsonValue) -> bool {
    let r = obj.as_u64();
    r.is_some()
}
