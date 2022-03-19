extern crate atty;
#[cfg(feature = "c_fixed_string")]
extern crate c_fixed_string;
extern crate chrono;
extern crate dateparser;
extern crate derive_more;
#[cfg(feature = "flagset")]
extern crate flagset;
extern crate futures_util;
extern crate json;
extern crate indicatif;
#[cfg(feature = "int-enum")]
extern crate int_enum;
#[macro_use]
extern crate lazy_static;
#[cfg(all(feature = "link-cplusplus", target_env = "gnu"))]
extern crate link_cplusplus;
extern crate tokio;
extern crate regex;
extern crate reqwest;
extern crate urlparse;
#[cfg(feature = "utf16string")]
extern crate utf16string;
extern crate xml;

#[cfg(feature = "avdict")]
mod _avdict;
#[cfg(feature = "exif")]
mod _exif;
#[cfg(feature = "ugoira")]
mod _ugoira;
mod author_name_filter;
#[cfg(feature = "avdict")]
mod avdict;
mod cookies;
mod data;
mod download;
mod dur;
#[cfg(feature = "exif")]
mod exif;
/// Used to extend some thirdparty library
mod ext;
mod i18n;
mod list;
mod opthelper;
mod opts;
mod parser;
mod pixiv_link;
mod pixiv_web;
mod retry_interval;
mod settings;
mod settings_list;
mod stdext;
#[cfg(feature = "ugoira")]
mod ugoira;
mod utils;
mod webclient;

use i18n::gettext;
use opts::Command;
use opts::CommandOpts;
use opts::ConfigCommand;
use settings::SettingStore;

pub struct Main {
    pub cmd: Option<CommandOpts>,
    pub settings: Option<SettingStore>,
}

impl Main {
    pub fn deal_config_cmd(&mut self) -> i32 {
        let cmd = self.cmd.as_ref().unwrap();
        let subcmd = cmd.config_cmd.as_ref().unwrap();
        match subcmd {
            ConfigCommand::Fix => {
                let s = self.settings.as_ref().unwrap();
                let conf = cmd.config();
                if conf.is_some() {
                    if s.save(conf.as_ref().unwrap()) {
                        0
                    } else {
                        println!(
                            "{} {}",
                            gettext("Failed to save config file:"),
                            conf.as_ref().unwrap()
                        );
                        1
                    }
                } else {
                    0
                }
            }
            ConfigCommand::Help => {
                let s = self.settings.as_ref().unwrap();
                println!("{}", gettext("All available settings:"));
                s.basic.print_help();
                0
            }
        }
    }

    pub fn new() -> Self {
        Self {
            cmd: None,
            settings: None,
        }
    }

    pub fn run(&mut self) -> i32 {
        self.cmd = opts::parse_cmd();
        if self.cmd.is_none() {
            return 0;
        }
        let cmd = self.cmd.as_ref().unwrap();
        self.settings = Some(SettingStore::default());
        match cmd.config() {
            Some(conf) => {
                let fix_invalid = if cmd.cmd == Command::Config
                    && cmd.config_cmd.as_ref().unwrap() == ConfigCommand::Fix
                {
                    true
                } else {
                    false
                };
                let r = self.settings.as_mut().unwrap().read(&conf, fix_invalid);
                if !r {
                    println!("{} {}", gettext("Can not read config file:"), conf.as_str());
                    return 1;
                }
            }
            None => {}
        }
        match cmd.cmd {
            Command::Config => {
                self.deal_config_cmd();
            }
            Command::Download => {
                return self.download();
            }
        }
        0
    }
}

#[tokio::main]
async fn main() {
    let mut m = Main::new();
    std::process::exit(m.run());
}
