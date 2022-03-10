use crate::gettext;
use crate::settings_list::get_settings_list;
use json::JsonValue;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::{remove_file, File};
use std::io::{Read, Write};
use std::path::Path;

/// Json value type
#[derive(Clone, Copy, PartialEq)]
pub enum JsonValueType {
    Str,
    Number,
    Boolean,
    Object,
    Array,
    Multiple,
}

impl JsonValueType {
    pub fn to_str(&self) -> &'static str {
        match self {
            JsonValueType::Str => "String",
            JsonValueType::Number => "Number",
            JsonValueType::Boolean => "Boolean",
            JsonValueType::Object => "Object",
            JsonValueType::Array => "Array",
            JsonValueType::Multiple => gettext("Multiple type"),
        }
    }
}

impl Debug for JsonValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl Display for JsonValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// An callback to check if a json value is valid
pub type SettingDesCallback = fn(obj: &JsonValue) -> bool;

/// An object to describe a setting
#[derive(Clone)]
pub struct SettingDes {
    /// The name of the setting
    _name: String,
    /// The description of the setting
    _description: String,
    /// The type of the setting
    _type: JsonValueType,
    /// The callback function of the setting
    _fun: Option<SettingDesCallback>,
}

impl SettingDes {
    /// Create a new setting description
    pub fn new(
        name: &str,
        description: &str,
        typ: JsonValueType,
        callback: Option<SettingDesCallback>,
    ) -> Option<SettingDes> {
        if (typ == JsonValueType::Array
            || typ == JsonValueType::Object
            || typ == JsonValueType::Multiple)
            && callback.is_none()
        {
            return None;
        }
        Some(SettingDes {
            _name: String::from(name),
            _description: String::from(description),
            _type: typ.clone(),
            _fun: callback,
        })
    }

    pub fn name(&self) -> &str {
        self._name.as_str()
    }

    pub fn description(&self) -> &str {
        self._description.as_str()
    }

    pub fn type_name(&self) -> &'static str {
        self._type.to_str()
    }
    /// Check if a value is valid
    pub fn is_vaild_value(&self, value: &JsonValue) -> bool {
        if self._type == JsonValueType::Array {
            if value.is_array() {
                return self._fun.unwrap()(&value);
            }
        } else if self._type == JsonValueType::Boolean {
            if value.is_boolean() {
                return true;
            }
        } else if self._type == JsonValueType::Multiple {
            return self._fun.unwrap()(&value);
        } else if self._type == JsonValueType::Number {
            if value.is_number() {
                match self._fun {
                    Some(fun) => {
                        return fun(&value);
                    }
                    None => {
                        return true;
                    }
                }
            }
        } else if self._type == JsonValueType::Object {
            if value.is_object() {
                return self._fun.unwrap()(&value);
            }
        } else if self._type == JsonValueType::Str {
            if value.is_string() {
                match self._fun {
                    Some(fun) => {
                        return fun(&value);
                    }
                    None => {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}

impl Debug for SettingDes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "SettingDes {{ name: {}, description: {}, type: {} }}",
            self._name, self._description, self._type
        )
    }
}

/// Store a list of settings
#[derive(Clone, Debug)]
pub struct SettingDesStore {
    list: Vec<SettingDes>,
}

impl SettingDesStore {
    pub fn new(list: Vec<SettingDes>) -> SettingDesStore {
        SettingDesStore { list }
    }

    pub fn check_valid(&self, key: &str, value: &JsonValue) -> Option<bool> {
        for i in self.list.iter() {
            if i.name() == key {
                return Some(i.is_vaild_value(value));
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn print_help(&self) {
        let mut s = String::from("");
        for i in self.list.iter() {
            let mut t = format!("{}: {}", i.name(), i.type_name());
            if t.len() >= 20 {
                t += "\t";
            } else {
                t += " ".repeat(20 - t.len()).as_str();
            }
            t += i.description();
            if s.len() > 0 {
                s += "\n";
            }
            s += t.as_str();
        }
        println!("{}", s);
    }
}

impl Default for SettingDesStore {
    fn default() -> Self {
        Self {
            list: get_settings_list(),
        }
    }
}

/// An settings option
#[derive(Clone)]
pub struct SettingOpt {
    _name: String,
    _value: JsonValue,
}

impl SettingOpt {
    pub fn new(name: String, value: JsonValue) -> SettingOpt {
        SettingOpt {
            _name: name.clone(),
            _value: value.clone(),
        }
    }

    pub fn name(&self) -> String {
        self._name.clone()
    }

    pub fn value(&self) -> JsonValue {
        self._value.clone()
    }
}

impl Debug for SettingOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self._value, f)
    }
}

#[derive(Clone, Debug)]
pub struct SettingJar {
    pub settings: HashMap<String, SettingOpt>,
}

impl SettingJar {
    pub fn new() -> SettingJar {
        SettingJar {
            settings: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: &str, opt: JsonValue) {
        self.settings
            .insert(String::from(key), SettingOpt::new(String::from(key), opt));
    }

    pub fn clear(&mut self) {
        self.settings.clear();
    }

    pub fn get(&self, key: &str) -> Option<JsonValue> {
        if self.settings.contains_key(key) {
            return Some(self.settings.get(key).unwrap().value());
        }
        None
    }

    pub fn have(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }

    pub fn to_json(&self) -> Option<JsonValue> {
        let mut v = JsonValue::new_object();
        for (_, val) in self.settings.iter() {
            match v.insert(val.name().as_str(), val.value()) {
                Ok(_) => {}
                Err(_) => {
                    println!("{}", gettext("Can not insert setting to JSON object."));
                    return None;
                }
            }
        }
        Some(v)
    }
}

#[derive(Clone, Debug)]
pub struct SettingStore {
    pub basic: SettingDesStore,
    pub data: SettingJar,
}

impl SettingStore {
    pub fn new(list: Vec<SettingDes>) -> Self {
        Self {
            basic: SettingDesStore::new(list),
            data: SettingJar::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<JsonValue> {
        self.data.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(obj) => { obj.as_bool() }
            None => { None }
        }
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        let obj = self.data.get(key);
        if obj.is_none() {
            return None;
        }
        let obj = obj.unwrap();
        if !obj.is_string() {
            None
        } else {
            Some(String::from(obj.as_str().unwrap()))
        }
    }

    pub fn have(&self, key: &str) -> bool {
        self.data.have(key)
    }

    pub fn have_bool(&self, key: &str) -> bool {
        match self.data.get(key) {
            Some(obj) => { obj.is_boolean() }
            None => { false }
        }
    }

    pub fn have_str(&self, key: &str) -> bool {
        let obj = self.data.get(key);
        if obj.is_none()  {
            return false;
        }
        let obj = obj.unwrap();
        obj.is_string()
    }

    pub fn read(&mut self, file_name: &str, fix_invalid: bool) -> bool {
        self.data.clear();
        let path = Path::new(file_name);
        if !path.exists() {
            return false;
        }
        let re = File::open(path);
        if re.is_err() {
            println!("{}", re.unwrap_err());
            return false;
        }
        let mut f = re.unwrap();
        let mut s = String::from("");
        let r = f.read_to_string(&mut s);
        match r {
            Ok(le) => {
                if le == 0 {
                    if !fix_invalid {
                        println!("{}", gettext("Settings file is empty."));
                        return false;
                    }
                    return true;
                }
            }
            Err(_) => {
                println!("{}", gettext("Can not read from settings file."));
                return false;
            }
        }
        let re = json::parse(s.as_str());
        match re {
            Ok(_) => {}
            Err(_) => {
                if !fix_invalid {
                    println!("{}", gettext("Can not parse settings file."));
                    return false;
                }
                return true;
            }
        }
        let obj = re.unwrap();
        if !obj.is_object() {
            if !fix_invalid {
                println!("{}", gettext("Unknown settings file."));
                return false;
            }
            return true;
        }
        for (key, o) in obj.entries() {
            let re = self.basic.check_valid(key, o);
            match re {
                Some(re) => {
                    if !re {
                        if !fix_invalid {
                            let s = gettext("\"<key>\" is invalid, you can use \"pixiv_downloader config fix\" to remove all invalid value.").replace("<key>", key);
                            println!("{}", s.as_str());
                            return false;
                        }
                    } else {
                        self.data.add(key, o.clone());
                    }
                }
                None => {
                    self.data.add(key, o.clone());
                }
            }
        }
        true
    }

    pub fn save(&self, file_name: &str) -> bool {
        let obj = self.data.to_json();
        if obj.is_none() {
            println!("{}", gettext("Can not convert settings to JSON object."));
            return false;
        }
        let obj = obj.unwrap();
        let s = json::stringify(obj);
        let path = Path::new(file_name);
        if path.exists() {
            match remove_file(path) {
                Ok(_) => {}
                Err(e) => {
                    println!("{} {}", gettext("Failed to remove file:"), e);
                    return false;
                }
            }
        }
        let r = File::create(path);
        if r.is_err() {
            println!("{} {}", gettext("Failed to create file:"), r.unwrap_err());
            return false;
        }
        let mut f = r.unwrap();
        let r = f.write(s.as_bytes());
        if r.is_err() {
            println!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
            return false;
        }
        let r = f.flush();
        if r.is_err() {
            println!("{} {}", gettext("Failed to flush file:"), r.unwrap_err());
            return false;
        }
        true
    }
}

impl Default for SettingStore {
    fn default() -> Self {
        Self::new(get_settings_list())
    }
}
