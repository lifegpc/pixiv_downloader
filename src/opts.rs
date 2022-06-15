extern crate getopts;

use crate::ext::use_or_not::UseOrNot;
use crate::gettext;
use crate::list::NonTailList;
use crate::pixiv_link::PixivID;
use crate::retry_interval::parse_retry_interval_from_str;
use crate::utils::check_file_exists;
use crate::utils::get_exe_path_else_current;
use getopts::HasArg;
use getopts::Options;
use std::env;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

/// Command Line command
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Do something for the config
    Config,
    /// Download an artwork
    Download,
    /// Already handled when parsing options, just need return 0.
    None,
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

#[derive(Clone, Debug)]
/// Command Line Options
pub struct CommandOpts {
    /// Command
    pub cmd: Command,
    /// IDs
    pub ids: Vec<PixivID>,
    /// Config location
    pub _config: Option<String>,
    /// Config command
    pub config_cmd: Option<ConfigCommand>,
    /// The location of cookies file
    pub cookies: Option<String>,
    /// The language of translated tags
    pub language: Option<String>,
    /// Verbose logging
    pub verbose: bool,
    /// Whether to overwrite file
    pub overwrite: Option<bool>,
    /// Max retry count.
    pub retry: Option<i64>,
    /// Retry interval
    pub retry_interval: Option<NonTailList<Duration>>,
    /// Use data from webpage first
    pub use_webpage: bool,
    #[cfg(feature = "exif")]
    /// Add/Update exif information to image files even when overwrite are disabled
    pub update_exif: bool,
    /// Whether to enable progress bar
    pub use_progress_bar: Option<UseOrNot>,
    /// Whether to download multiple images at the same time
    pub download_multiple_images: Option<bool>,
    /// Max retry count when downloading failed.
    pub download_retry: Option<i64>,
    /// Retry interval when downloading files.
    pub download_retry_interval: Option<NonTailList<Duration>>,
    /// Whether to enable multiple threads download.
    pub multiple_threads_download: Option<bool>,
    /// Max retry count of each part when downloading in multiple thread mode.
    pub download_part_retry: Option<i64>,
}

impl CommandOpts {
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd,
            ids: Vec::new(),
            _config: None,
            config_cmd: None,
            cookies: None,
            language: None,
            verbose: false,
            overwrite: None,
            retry: None,
            retry_interval: None,
            use_webpage: false,
            #[cfg(feature = "exif")]
            update_exif: false,
            use_progress_bar: None,
            download_multiple_images: None,
            download_retry: None,
            download_retry_interval: None,
            multiple_threads_download: None,
            download_part_retry: None,
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
{} download/d [options] <id/url> [<id/url>]  {}
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

/// Prase [bool] from string
pub fn parse_bool<T: AsRef<str>>(s: Option<T>) -> Result<Option<bool>, String> {
    let tmp = match s {
        Some(s) => Some(s.as_ref().to_lowercase()),
        None => None,
    };
    match tmp {
        Some(t) => {
            if t == "true" {
                Ok(Some(true))
            } else if t == "false" {
                Ok(Some(false))
            } else if t == "yes" {
                Ok(Some(true))
            } else if t == "no" {
                Ok(Some(false))
            } else {
                Err(format!("{} {}", gettext("Invalid boolean value:"), t))
            }
        }
        None => Ok(None),
    }
}

/// Prase [i64] from string
pub fn parse_i64<T: AsRef<str>>(s: Option<T>) -> Result<Option<i64>, ParseIntError> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            let s = s.trim();
            let c = s.parse::<i64>()?;
            Ok(Some(c))
        }
        None => Ok(None),
    }
}

/// Parse optional option
/// * `opts` - The result of options. See [getopts::Matches].
/// * `key` - The key of the option.
/// * `default` - The value if option is present but the data is not obtained.
/// * `callback` - The function to process the obtained data.
pub fn parse_optional_opt<T, F, E>(
    opts: &getopts::Matches,
    key: &str,
    default: T,
    callback: F,
) -> Result<Option<T>, E>
where
    F: Fn(Option<String>) -> Result<Option<T>, E>,
{
    if !opts.opt_present(key) {
        return Ok(None);
    }
    let s = opts.opt_str(key);
    if s.is_none() {
        return Ok(Some(default));
    }
    callback(s)
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
    opts.optflag("v", "verbose", gettext("Verbose logging."));
    opts.optflag("y", "yes", gettext("Overwrite existing file."));
    opts.optflag("n", "no", gettext("Skip overwrite existing file."));
    opts.optopt(
        "r",
        "retry",
        gettext("Max retry count if request failed."),
        "COUNT",
    );
    opts.optopt(
        "",
        "retry-interval",
        gettext("The interval (in seconds) between two retries."),
        "LIST",
    );
    opts.optflag("", "use-webpage", gettext("Use data from webpage first."));
    #[cfg(feature = "exif")]
    opts.optflag(
        "",
        "update-exif",
        gettext("Add/Update exif information to image files even when overwrite are disabled."),
    );
    opts.optopt(
        "",
        "use-progress-bar",
        gettext("Whether to enable progress bar."),
        "yes/no/auto",
    );
    opts.opt(
        "",
        "download-multiple-images",
        format!(
            "{} ({} {})",
            gettext("Download multiple images at the same time."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "",
        "download-retry",
        gettext("Max retry count if download failed."),
        "COUNT",
    );
    opts.optopt(
        "",
        "download-retry-interval",
        gettext("The interval (in seconds) between two retries when downloading files."),
        "LIST",
    );
    opts.opt(
        "",
        "multiple-threads-download",
        format!(
            "{} ({} {})",
            gettext("Whether to enable multiple threads download."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "",
        "download-part-retry",
        gettext("Max retry count of each part when downloading in multiple thread mode."),
        "COUNT",
    );
    let result = match opts.parse(&argv[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("{}", err.to_string());
            return None;
        }
    };
    if result.opt_present("h") || result.free.len() == 0 {
        print_usage(&argv[0], &opts);
        return Some(CommandOpts::new(Command::None));
    }
    let cmd = &result.free[0];
    let mut re = if cmd == "download" || cmd == "d" {
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
            let mut ids = Vec::new();
            for url in result.free.iter().skip(1) {
                let id = PixivID::parse(url);
                if id.is_none() {
                    println!("{} {}", gettext("Failed to parse ID:"), url);
                    return None;
                }
                ids.push(id.unwrap());
            }
            if ids.is_empty() {
                println!("{}", gettext("No URL or ID specified."));
                print_usage(&argv[0], &opts);
                return None;
            }
            re.as_mut().unwrap().ids = ids;
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
        Command::None => {}
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
    re.as_mut().unwrap().verbose = result.opt_present("verbose");
    let yes = result.opt_present("yes");
    let no = result.opt_present("no");
    re.as_mut().unwrap().overwrite = if yes && no {
        if result.opt_positions("yes").last().unwrap() > result.opt_positions("no").last().unwrap()
        {
            Some(true)
        } else {
            Some(false)
        }
    } else if yes {
        Some(true)
    } else if no {
        Some(false)
    } else {
        None
    };
    match parse_i64(result.opt_str("retry")) {
        Ok(r) => {
            re.as_mut().unwrap().retry = r;
        }
        Err(e) => {
            println!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    if result.opt_present("retry-interval") {
        let s = result.opt_str("retry-interval").unwrap();
        let r = parse_retry_interval_from_str(s.as_str());
        if r.is_err() {
            println!(
                "{} {}",
                gettext("Failed to parse retry interval:"),
                r.unwrap_err()
            );
            return None;
        }
        re.as_mut().unwrap().retry_interval = Some(r.unwrap());
    }
    re.as_mut().unwrap().use_webpage = result.opt_present("use-webpage");
    #[cfg(feature = "exif")]
    {
        re.as_mut().unwrap().update_exif = result.opt_present("update-exif");
    }
    if result.opt_present("use-progress-bar") {
        let s = result.opt_str("use-progress-bar").unwrap();
        let r = UseOrNot::from_str(s.as_str());
        if r.is_err() {
            println!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "use-progress-bar")
                    .as_str(),
                r.unwrap_err()
            );
            return None;
        }
        re.as_mut().unwrap().use_progress_bar = Some(r.unwrap());
    }
    match parse_optional_opt(&result, "download-multiple-images", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().download_multiple_images = b,
        Err(e) => {
            println!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "download-multiple-images")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_i64(result.opt_str("download-retry")) {
        Ok(r) => {
            re.as_mut().unwrap().download_retry = r;
        }
        Err(e) => {
            println!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    if result.opt_present("download-retry-interval") {
        let s = result.opt_str("download-retry-interval").unwrap();
        let r = parse_retry_interval_from_str(s.as_str());
        if r.is_err() {
            println!(
                "{} {}",
                gettext("Failed to parse retry interval:"),
                r.unwrap_err()
            );
            return None;
        }
        re.as_mut().unwrap().download_retry_interval = Some(r.unwrap());
    }
    match parse_i64(result.opt_str("download-part-retry")) {
        Ok(r) => {
            re.as_mut().unwrap().download_part_retry = r;
        }
        Err(e) => {
            println!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    match parse_optional_opt(&result, "multiple-threads-download", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().multiple_threads_download = b,
        Err(e) => {
            println!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "multiple-threads-download")
                    .as_str(),
                e
            );
            return None;
        }
    }
    re
}

impl Default for CommandOpts {
    fn default() -> Self {
        Self::new(Command::None)
    }
}
