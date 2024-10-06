use crate::ext::use_or_not::UseOrNot;
use crate::gettext;
use crate::list::NonTailList;
use crate::pixiv_link::PixivID;
use crate::retry_interval::parse_retry_interval_from_str;
use crate::ugoira::X264Profile;
use crate::utils::check_file_exists;
use crate::utils::get_exe_path_else_current;
use getopts::HasArg;
use getopts::Options;
use std::env;
#[cfg(feature = "server")]
use std::net::SocketAddr;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::num::TryFromIntError;
#[cfg(feature = "docker")]
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Command Line command
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Do something for the config
    Config,
    /// Download an artwork
    Download,
    /// Download files from urls
    DownloadFile,
    #[cfg(feature = "server")]
    /// Run as a server
    Server,
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
    /// Whether to download multiple files at the same time
    pub download_multiple_files: Option<bool>,
    /// Max retry count when downloading failed.
    pub download_retry: Option<i64>,
    /// Retry interval when downloading files.
    pub download_retry_interval: Option<NonTailList<Duration>>,
    /// Whether to enable multiple threads download.
    pub multiple_threads_download: Option<bool>,
    /// Max retry count of each part when downloading in multiple thread mode.
    pub download_part_retry: Option<i64>,
    /// The maximum threads when downloading file.
    pub max_threads: Option<u64>,
    /// The size of the each part when downloading file.
    pub part_size: Option<u32>,
    #[cfg(feature = "server")]
    /// Server listen address
    pub server: Option<SocketAddr>,
    /// maximum number of tasks to download files at the same time
    pub max_download_tasks: Option<usize>,
    /// Whether to download multiple posts/artworks at the same time.
    pub download_multiple_posts: Option<bool>,
    /// The maximum number of tasks to download posts/artworks at the same time.
    pub max_download_post_tasks: Option<usize>,
    /// Whether to force yuv420p as output pixel format when converting ugoira(GIF) to video.
    pub force_yuv420p: Option<bool>,
    /// The x264 profile when converting ugoira(GIF) to video.
    pub x264_profile: Option<X264Profile>,
    /// The base directory to save downloaded files.
    pub download_base: Option<String>,
    pub user_agent: Option<String>,
    /// Urls want to download
    pub urls: Option<Vec<String>>,
    /// The Constant Rate Factor when converting ugoira(GIF) to video.
    pub x264_crf: Option<f32>,
    pub ugoira_max_fps: Option<f32>,
    pub fanbox_page_number: Option<bool>,
    /// Pixiv's refresh token. Used to login.
    pub refresh_token: Option<String>,
    /// Whether to use Pixiv APP API first.
    pub use_app_api: Option<bool>,
    /// Whether to use description from Web API when description from APP API is empty.
    pub use_web_description: Option<bool>,
    /// Whether to add artworks to pixiv's history. Only works for APP API.
    pub add_history: Option<bool>,
    #[cfg(feature = "server")]
    /// The maximum number of push tasks running at the same time.
    pub push_task_max_count: Option<usize>,
    #[cfg(feature = "server")]
    /// The maximum number of tasks to push to client at the same time.
    pub push_task_max_push_count: Option<usize>,
    #[cfg(feature = "server")]
    /// Whether to prevent to run push task.
    pub disable_push_task: bool,
    /// The path to ugoira cli executable.
    pub ugoira: Option<String>,
    #[cfg(feature = "ugoira")]
    /// Whether to use ugoira cli.
    pub ugoira_cli: Option<bool>,
    /// Set a timeout in milliseconds for only the connect phase of a client.
    pub connect_timeout: Option<u64>,
    /// Set request timeout in milliseconds.
    /// The timeout is applied from when the request starts connecting until the response body
    /// has finished. Not used for downloader.
    pub client_timeout: Option<u64>,
    /// The path to ffprobe executable.
    pub ffprobe: Option<String>,
    /// The path to ffmpeg executable.
    pub ffmpeg: Option<String>,
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
            download_multiple_files: None,
            download_retry: None,
            download_retry_interval: None,
            multiple_threads_download: None,
            download_part_retry: None,
            max_threads: None,
            part_size: None,
            #[cfg(feature = "server")]
            server: None,
            max_download_tasks: None,
            download_multiple_posts: None,
            max_download_post_tasks: None,
            force_yuv420p: None,
            x264_profile: None,
            download_base: None,
            user_agent: None,
            urls: None,
            x264_crf: None,
            ugoira_max_fps: None,
            fanbox_page_number: None,
            refresh_token: None,
            use_app_api: None,
            use_web_description: None,
            add_history: None,
            #[cfg(feature = "server")]
            push_task_max_count: None,
            #[cfg(feature = "server")]
            push_task_max_push_count: None,
            #[cfg(feature = "server")]
            disable_push_task: false,
            ugoira: None,
            #[cfg(feature = "ugoira")]
            ugoira_cli: None,
            connect_timeout: None,
            client_timeout: None,
            ffprobe: None,
            ffmpeg: None,
        }
    }

    pub fn new_with_command<S: AsRef<str> + ?Sized>(cmd: &S) -> Option<Self> {
        let cmd = cmd.as_ref();
        if cmd == "download" || cmd == "d" {
            return Some(CommandOpts::new(Command::Download));
        }
        if cmd == "config" {
            return Some(CommandOpts::new(Command::Config));
        }
        #[cfg(feature = "server")]
        if cmd == "server" || cmd == "s" {
            return Some(CommandOpts::new(Command::Server));
        }
        if cmd == "download-file" || cmd == "df" {
            return Some(CommandOpts::new(Command::DownloadFile));
        }
        None
    }

    pub fn config(&self) -> Option<String> {
        if self._config.is_some() {
            if check_file_exists(&self._config.as_ref().unwrap()) {
                self._config.clone()
            } else {
                log::error!(
                    "{}",
                    gettext("Warning: The specified config file not found.")
                );
                None
            }
        } else {
            #[cfg(feature = "docker")]
            {
                let pb = PathBuf::from("/app/data/config.json");
                if pb.exists() {
                    return Some(String::from(pb.to_str().unwrap()));
                }
            }
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

#[allow(unused_mut)]
pub fn print_usage(prog: &str, opts: &Options) {
    let mut brief = format!(
        "{}
{} download/d [options] <id/url> [<id/url>]  {}
{} config fix [options] {}
{} config help [options] {}
{} download-file/df [options] <url> [<url>] {}",
        gettext("Usage:"),
        prog,
        gettext("Download an artwork"),
        prog,
        gettext("Fix the config file"),
        prog,
        gettext("Print all available settings"),
        prog,
        gettext("Download files from url"),
    );
    #[cfg(feature = "server")]
    {
        brief += format!(
            "\n{} server/s [options] [address]  {}",
            prog,
            gettext("Run as a server")
        )
        .as_str();
    }
    log::error!("{}", opts.usage(brief.as_str()));
}

/// Error when parsing size
#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum ParseSizeError {
    /// Failed to parse size.
    ParseSize(parse_size::Error),
    /// The size is too big.
    Overflow(TryFromIntError),
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

/// Parse [f32] from string
pub fn parse_f32<T: AsRef<str>>(s: Option<T>) -> Result<Option<f32>, ParseFloatError> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            let s = s.trim();
            let c = s.parse::<f32>()?;
            Ok(Some(c))
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

/// Prase size as [u32] from string
pub fn parse_u32_size<T: AsRef<str>>(s: Option<T>) -> Result<Option<u32>, ParseSizeError> {
    match s {
        Some(s) => {
            let s = parse_size::parse_size(s.as_ref())?;
            Ok(Some(u32::try_from(s)?))
        }
        None => Ok(None),
    }
}

/// Prase [u64] from string
pub fn parse_u64<T: AsRef<str>>(s: Option<T>) -> Result<Option<u64>, ParseIntError> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            let s = s.trim();
            let c = s.parse::<u64>()?;
            Ok(Some(c))
        }
        None => Ok(None),
    }
}

/// Prase Non Zero [u64] from string
pub fn parse_non_zero_u64<T: AsRef<str>>(s: Option<T>) -> Result<Option<u64>, ParseIntError> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            let s = s.trim();
            let c = s.parse::<std::num::NonZeroU64>()?;
            Ok(Some(c.get()))
        }
        None => Ok(None),
    }
}

pub fn parse_nonempty_usize<T: AsRef<str>>(s: Option<T>) -> Result<Option<usize>, ParseIntError> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            let s = s.trim();
            let c = s.parse::<std::num::NonZeroUsize>()?;
            Ok(Some(c.get()))
        }
        None => Ok(None),
    }
}

pub fn parse_x264_profile<S: AsRef<str>>(
    s: Option<S>,
) -> Result<Option<X264Profile>, &'static str> {
    match s {
        Some(s) => {
            let s = s.as_ref();
            Ok(Some(X264Profile::from_str(s)?))
        }
        None => Ok(Some(X264Profile::default())),
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
        "download-multiple-files",
        format!(
            "{} ({} {})",
            gettext("Download multiple files at the same time."),
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
    opts.optopt(
        "m",
        "max-threads",
        gettext("The maximum threads when downloading file."),
        "COUNT",
    );
    opts.optopt(
        "k",
        "part-size",
        gettext("The size of the each part when downloading file."),
        "SIZE",
    );
    opts.opt(
        "",
        "max-download-tasks",
        format!(
            "{} ({} {})",
            gettext("The maximum number of tasks to download files at the same time."),
            gettext("Default:"),
            "5"
        )
        .as_str(),
        "COUNT",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "download-multiple-posts",
        format!(
            "{} ({} {})",
            gettext("Download multiple posts/artworks at the same time."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "max-download-post-tasks",
        format!(
            "{} ({} {})",
            gettext("The maximum number of tasks to download posts/artworks at the same time."),
            gettext("Default:"),
            3
        )
        .as_str(),
        "COUNT",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "force-yuv420p",
        format!(
            "{} ({} {})",
            gettext("Force yuv420p as output pixel format when converting ugoira(GIF) to video."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "x264-profile",
        format!(
            "{} ({} {})",
            gettext("The x264 profile when converting ugoira(GIF) to video."),
            gettext("Default:"),
            "auto"
        )
        .as_str(),
        "PROFILE",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "d",
        "download-base",
        gettext("The base directory to download files."),
        "DIR",
    );
    opts.optopt("", "user-agent", gettext("The User-Agent header."), "UA");
    opts.opt(
        "",
        "x264-crf",
        gettext("The Constant Rate Factor when converting ugoira(GIF) to video."),
        "float",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "ugoira-max-fps",
        format!(
            "{} ({} {})",
            gettext("The max fps when converting ugoira(GIF) to video."),
            gettext("Default:"),
            "60"
        )
        .as_str(),
        "float",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "fanbox-page-number",
        gettext("Use page number for pictures' file name in fanbox."),
        "BOOLEAN",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "",
        "refresh-token",
        gettext("Pixiv's refresh token. Used to login."),
        "TOKEN",
    );
    opts.opt(
        "",
        "use-app-api",
        format!(
            "{} ({} {})",
            gettext("Whether to use Pixiv APP API first."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "use-web-description",
        format!(
            "{} ({} {})",
            gettext(
                "Whether to use description from Web API when description from APP API is empty."
            ),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.opt(
        "",
        "add-history",
        format!(
            "{} ({} {})",
            gettext("Whether to add artworks to pixiv's history. Only works for APP API."),
            gettext("Default:"),
            "yes"
        )
        .as_str(),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    #[cfg(feature = "server")]
    opts.optopt(
        "",
        "push-task-max-count",
        gettext("The maximum number of push tasks running at the same time."),
        "COUNT",
    );
    #[cfg(feature = "server")]
    opts.optopt(
        "",
        "push-task-max-push-count",
        gettext("The maximum number of tasks to push to client at the same time."),
        "COUNT",
    );
    #[cfg(feature = "server")]
    opts.optflag(
        "",
        "disable-push-task",
        gettext("Prevent to run push task."),
    );
    opts.optopt(
        "",
        "ugoira",
        gettext("The path to ugoira cli executable."),
        "PATH",
    );
    #[cfg(feature = "ugoira")]
    opts.opt(
        "",
        "ugoira-cli",
        &format!(
            "{} ({} {})",
            gettext("Whether to use ugoira cli."),
            gettext("Default:"),
            "yes"
        ),
        "yes/no",
        HasArg::Maybe,
        getopts::Occur::Optional,
    );
    opts.optopt(
        "",
        "connect-timeout",
        gettext("Set a timeout in milliseconds for only the connect phase of a client."),
        "TIME",
    );
    opts.optopt("", "client-timeout", gettext("Set request timeout in milliseconds. The timeout is applied from when the request starts connecting until the response body has finished. Not used for downloader."), "TIME");
    opts.optopt(
        "",
        "ffprobe",
        gettext("The path to ffprobe executable."),
        "PATH",
    );
    opts.optopt(
        "",
        "ffmpeg",
        gettext("The path to ffmpeg executable."),
        "PATH",
    );
    let result = match opts.parse(&argv[1..]) {
        Ok(m) => m,
        Err(err) => {
            log::error!("{}", err.to_string());
            return None;
        }
    };
    if result.opt_present("h") || result.free.len() == 0 {
        print_usage(&argv[0], &opts);
        return Some(CommandOpts::new(Command::None));
    }
    let mut re = CommandOpts::new_with_command(&result.free[0]);
    if re.is_none() {
        log::error!("{}", gettext("Unknown command."));
        print_usage(&argv[0], &opts);
        return None;
    }
    match re.as_ref().unwrap().cmd {
        Command::Download => {
            let mut ids = Vec::new();
            for url in result.free.iter().skip(1) {
                let id = PixivID::parse(url);
                if id.is_none() {
                    log::error!("{} {}", gettext("Failed to parse ID:"), url);
                    return None;
                }
                ids.push(id.unwrap());
            }
            if ids.is_empty() {
                log::error!("{}", gettext("No URL or ID specified."));
                print_usage(&argv[0], &opts);
                return None;
            }
            re.as_mut().unwrap().ids = ids;
        }
        Command::Config => {
            if result.free.len() < 2 {
                log::error!("{}", gettext("No detailed command specified."));
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
                log::error!("{}", gettext("Unknown config subcommand."));
                print_usage(&argv[0], &opts);
                return None;
            }
        }
        #[cfg(feature = "server")]
        Command::Server => {
            if result.free.len() >= 2 {
                let address = &result.free[1];
                match SocketAddr::from_str(address) {
                    Ok(address) => re.as_mut().unwrap().server = Some(address),
                    Err(e) => {
                        log::error!("{} {}", gettext("Failed to parse address:"), e);
                        return None;
                    }
                }
            }
        }
        Command::DownloadFile => {
            let mut urls = Vec::new();
            for url in result.free.iter().skip(1) {
                urls.push(url.to_owned());
            }
            if urls.is_empty() {
                log::error!("{}", gettext("No URL specified."));
                print_usage(&argv[0], &opts);
                return None;
            }
            re.as_mut().unwrap().urls.replace(urls);
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
            log::error!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    if result.opt_present("retry-interval") {
        let s = result.opt_str("retry-interval").unwrap();
        let r = parse_retry_interval_from_str(s.as_str());
        if r.is_err() {
            log::error!(
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
            log::error!(
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
    match parse_optional_opt(&result, "download-multiple-files", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().download_multiple_files = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "download-multiple-files")
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
            log::error!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    if result.opt_present("download-retry-interval") {
        let s = result.opt_str("download-retry-interval").unwrap();
        let r = parse_retry_interval_from_str(s.as_str());
        if r.is_err() {
            log::error!(
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
            log::error!("{} {}", gettext("Failed to parse retry count:"), e);
            return None;
        }
    }
    match parse_optional_opt(&result, "multiple-threads-download", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().multiple_threads_download = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "multiple-threads-download")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_u64(result.opt_str("max-threads")) {
        Ok(r) => {
            re.as_mut().unwrap().max_threads = r;
        }
        Err(e) => {
            log::error!("{} {}", gettext("Failed to parse max threads:"), e);
            return None;
        }
    }
    match parse_u32_size(result.opt_str("part-size")) {
        Ok(r) => {
            re.as_mut().unwrap().part_size = r;
        }
        Err(e) => {
            log::error!("{} {}", gettext("Failed to parse part size:"), e);
            return None;
        }
    }
    match parse_optional_opt(&result, "max-download-tasks", 5, parse_nonempty_usize) {
        Ok(r) => re.as_mut().unwrap().max_download_tasks = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:").replace("<opt>", "max-download-tasks"),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "download-multiple-posts", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().download_multiple_posts = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "download-multiple-posts")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "max-download-post-tasks", 3, parse_nonempty_usize) {
        Ok(r) => re.as_mut().unwrap().max_download_post_tasks = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "max-download-post-tasks")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "force-yuv420p", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().force_yuv420p = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "force-yuv420p")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(
        &result,
        "x264-profile",
        X264Profile::default(),
        parse_x264_profile,
    ) {
        Ok(r) => re.as_mut().unwrap().x264_profile = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "x264-profile")
                    .as_str(),
                e
            );
            return None;
        }
    }
    re.as_mut().unwrap().download_base = result.opt_str("download-base");
    re.as_mut().unwrap().user_agent = result.opt_str("user-agent");
    match parse_optional_opt(&result, "x264-crf", -1f32, parse_f32) {
        Ok(r) => match r {
            Some(crf) => {
                if crf < -1f32 {
                    log::error!("{}", gettext("x264-crf can not less than -1."));
                    return None;
                } else {
                    re.as_mut().unwrap().x264_crf.replace(crf);
                }
            }
            None => {}
        },
        Err(e) => {
            log::error!(
                "{} {}",
                ("Failed to parse <opt>:")
                    .replace("<opt>", "x264-crf")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "ugoira-max-fps", 60f32, parse_f32) {
        Ok(r) => match r {
            Some(crf) => {
                if crf <= 0f32 || crf > 1000f32 {
                    log::error!(
                        "{}",
                        gettext("ugoira-max-fps can not less than 0 or greater than 1000.")
                    );
                    return None;
                } else {
                    re.as_mut().unwrap().ugoira_max_fps.replace(crf);
                }
            }
            None => {}
        },
        Err(e) => {
            log::error!(
                "{} {}",
                ("Failed to parse <opt>:")
                    .replace("<opt>", "ugoira-max-fps")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "fanbox-page-number", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().fanbox_page_number = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "fanbox-page-number")
                    .as_str(),
                e
            );
            return None;
        }
    }
    re.as_mut().unwrap().refresh_token = result.opt_str("refresh-token");
    match parse_optional_opt(&result, "use-app-api", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().use_app_api = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "use-app-api")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "use-web-description", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().use_web_description = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "use-web-description")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_optional_opt(&result, "add-history", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().add_history = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "add-history")
                    .as_str(),
                e
            );
            return None;
        }
    }
    #[cfg(feature = "server")]
    match parse_nonempty_usize(result.opt_str("push-task-max-count")) {
        Ok(r) => re.as_mut().unwrap().push_task_max_count = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "push-task-max-count")
                    .as_str(),
                e
            );
            return None;
        }
    }
    #[cfg(feature = "server")]
    match parse_nonempty_usize(result.opt_str("push-task-max-push-count")) {
        Ok(r) => re.as_mut().unwrap().push_task_max_push_count = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "push-task-max-push-count")
                    .as_str(),
                e
            );
            return None;
        }
    }
    #[cfg(feature = "server")]
    {
        re.as_mut().unwrap().disable_push_task = result.opt_present("disable-push-task");
    }
    re.as_mut().unwrap().ugoira = result.opt_str("ugoira");
    #[cfg(feature = "ugoira")]
    match parse_optional_opt(&result, "ugoira-cli", true, parse_bool) {
        Ok(b) => re.as_mut().unwrap().ugoira_cli = b,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "ugoira-cli")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_non_zero_u64(result.opt_str("connect-timeout")) {
        Ok(r) => re.as_mut().unwrap().connect_timeout = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "connect-timeout")
                    .as_str(),
                e
            );
            return None;
        }
    }
    match parse_non_zero_u64(result.opt_str("client-timeout")) {
        Ok(r) => re.as_mut().unwrap().client_timeout = r,
        Err(e) => {
            log::error!(
                "{} {}",
                gettext("Failed to parse <opt>:")
                    .replace("<opt>", "client-timeout")
                    .as_str(),
                e
            );
            return None;
        }
    }
    re.as_mut().unwrap().ffprobe = result.opt_str("ffprobe");
    re.as_mut().unwrap().ffmpeg = result.opt_str("ffmpeg");
    re
}

impl Default for CommandOpts {
    cfg_if! {
        if #[cfg(test)] {
            fn default() -> Self {
                let mut re = Self::new(Command::None);
                re.verbose = true;
                re
            }
        } else {
            fn default() -> Self {
                Self::new(Command::None)
            }
        }
    }
}
