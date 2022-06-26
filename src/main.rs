#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate lazy_static;
#[cfg(all(feature = "link-cplusplus", target_env = "gnu"))]
extern crate link_cplusplus;

#[cfg(feature = "avdict")]
mod _avdict;
#[cfg(feature = "exif")]
mod _exif;
#[cfg(feature = "ugoira")]
mod _ugoira;
#[cfg(feature = "avdict")]
/// A rust wrapper for [FFMPEG](https://ffmpeg.org/)'s [AVDictionary](https://ffmpeg.org/doxygen/trunk/group__lavu__dict.html)
mod avdict;
mod cookies;
mod data;
mod download;
mod downloader;
mod dur;
mod error;
#[cfg(feature = "exif")]
/// Used to read/modify image's exif data
mod exif;
/// Used to extend some thirdparty library
mod ext;
mod fanbox;
mod fanbox_api;
mod i18n;
mod list;
mod opt;
mod opthelper;
mod opts;
mod parser;
mod pixiv_link;
mod pixiv_web;
mod retry_interval;
mod settings;
mod settings_list;
#[cfg(feature = "ugoira")]
mod ugoira;
mod utils;
mod webclient;

use crate::i18n::gettext;
use crate::opthelper::get_helper;
use crate::opts::Command;
use crate::opts::CommandOpts;
use crate::opts::ConfigCommand;
use crate::settings::SettingStore;

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

    pub async fn run(&mut self) -> i32 {
        self.cmd = opts::parse_cmd();
        if self.cmd.is_none() {
            return 1;
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
        get_helper().update(cmd.clone(), self.settings.as_ref().unwrap().clone());
        match cmd.cmd {
            Command::Config => {
                self.deal_config_cmd();
            }
            Command::Download => {
                return self.download().await;
            }
            Command::None => {
                return 0;
            }
        }
        0
    }
}

#[tokio::main]
async fn main() {
    let mut m = Main::new();
    std::process::exit(m.run().await);
}
