extern crate getopts;

use crate::gettext;
use crate::utils::check_file_exists;
use crate::utils::get_exe_path_else_current;
use getopts::Options;
use std::env;

/// Command Line command
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Do something for the config
    Config,
    /// Download an artwork
    Download,
}

/// Subcommand for config
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigCommand {
    /// Fix the config file
    Fix,
    /// Print all available settings
    Help,
}

impl PartialEq<ConfigCommand> for &ConfigCommand {
    fn eq(&self, other: &ConfigCommand) -> bool {
        other == *self
    }
}

#[derive(Debug)]
/// Command Line Options
pub struct CommandOpts {
    /// Command
    pub cmd: Command,
    /// URLs
    pub urls: Vec<String>,
    /// Config location
    pub _config: Option<String>,
    /// Config command
    pub config_cmd: Option<ConfigCommand>,
    /// The location of cookies file
    pub cookies: Option<String>,
    /// The language of translated tags
    pub language: Option<String>,
}

impl CommandOpts {
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd,
            urls: Vec::new(),
            _config: None,
            config_cmd: None,
            cookies: None,
            language: None,
        }
    }

    pub fn config(&self) -> Option<String> {
        if self._config.is_some() {
            if check_file_exists(&self._config.as_ref().unwrap()) {
                self._config.clone()
            } else {
                println!(
                    "{}",
                    gettext("Warning: The specified config file not found.")
                );
                None
            }
        } else {
            let mut pb = get_exe_path_else_current();
            pb = pb.join("pixiv_downloader.json");
            if pb.exists() {
                return Some(String::from(pb.to_str().unwrap()));
            }
            if check_file_exists("config.json") {
                return Some(String::from("config.json"));
            }
            None
        }
    }
}

pub fn print_usage(prog: &str, opts: &Options) {
    let brief = format!(
        "{}
{} download [options] <url> [<url>]  {}
{} config fix [options] {}
{} config help [options] {}",
        gettext("Usage:"),
        prog,
        gettext("Download an artwork"),
        prog,
        gettext("Fix the config file"),
        prog,
        gettext("Print all available settings"),
    );
    println!("{}", opts.usage(brief.as_str()));
}

pub fn parse_cmd() -> Option<CommandOpts> {
    let argv: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", gettext("Print help message."));
    opts.optopt(
        "c",
        "config",
        gettext("The location of config file."),
        "FILE",
    );
    opts.optopt(
        "C",
        "cookies",
        gettext("The location of cookies file. Used for web API."),
        "FILE",
    );
    opts.optopt(
        "l",
        "language",
        gettext("The language of translated tags."),
        "LANG",
    );
    let result = match opts.parse(&argv[1..]) {
        Ok(m) => m,
        Err(err) => {
            panic!("{}", err.to_string())
        }
    };
    if result.opt_present("h") || result.free.len() == 0 {
        print_usage(&argv[0], &opts);
        return None;
    }
    let cmd = &result.free[0];
    let mut re = if cmd == "download" {
        Some(CommandOpts::new(Command::Download))
    } else if cmd == "config" {
        Some(CommandOpts::new(Command::Config))
    } else {
        None
    };
    if re.is_none() {
        println!("{}", gettext("Unknown command."));
        print_usage(&argv[0], &opts);
        return None;
    }
    match re.as_ref().unwrap().cmd {
        Command::Download => {
            let mut urls = Vec::new();
            for url in result.free.iter().skip(1) {
                urls.push(url.to_string());
            }
            if urls.is_empty() {
                println!("{}", gettext("No URL specified."));
                print_usage(&argv[0], &opts);
                return None;
            }
            re.as_mut().unwrap().urls = urls;
        }
        Command::Config => {
            if result.free.len() < 2 {
                println!("{}", gettext("No detailed command specified."));
                print_usage(&argv[0], &opts);
                return None;
            }
            let subcmd = &result.free[1];
            re.as_mut().unwrap().config_cmd = if subcmd == "fix" {
                Some(ConfigCommand::Fix)
            } else if subcmd == "help" {
                Some(ConfigCommand::Help)
            } else {
                None
            };
            if re.as_ref().unwrap().config_cmd.is_none() {
                println!("{}", gettext("Unknown config subcommand."));
                print_usage(&argv[0], &opts);
                return None;
            }
        }
    }
    if result.opt_present("config") {
        re.as_mut().unwrap()._config = Some(result.opt_str("config").unwrap());
    }
    if result.opt_present("cookies") {
        re.as_mut().unwrap().cookies = Some(result.opt_str("cookies").unwrap());
    }
    if result.opt_present("language") {
        re.as_mut().unwrap().language = Some(result.opt_str("language").unwrap());
    }
    re
}
