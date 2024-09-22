#[cfg(feature = "db")]
use crate::db::PixivDownloaderDbConfig;
use crate::ext::json::FromJson;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;
use crate::list::NonTailList;
use crate::opt::author_name_filter::AuthorNameFilter;
use crate::opt::header_map::HeaderMap;
use crate::opt::proxy::ProxyChain;
use crate::opt::size::parse_u32_size;
use crate::opt::use_progress_bar::UseProgressBar;
use crate::opts::CommandOpts;
use crate::retry_interval::parse_retry_interval_from_json;
#[cfg(feature = "server")]
use crate::server::cors::parse_cors_entries;
#[cfg(feature = "server")]
use crate::server::cors::CorsEntry;
use crate::settings::SettingStore;
use crate::ugoira::X264Profile;
use is_terminal::IsTerminal;
#[cfg(feature = "server")]
use std::net::IpAddr;
#[cfg(feature = "server")]
use std::net::Ipv4Addr;
#[cfg(feature = "server")]
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::time::Duration;

/// An sturct to access all available settings/command line switches
#[derive(Debug)]
pub struct OptHelper {
    /// Command Line Options
    opt: RwLock<CommandOpts>,
    /// Settings
    settings: RwLock<SettingStore>,
    default_retry_interval: NonTailList<Duration>,
    _author_name_filters: RwLock<Vec<AuthorNameFilter>>,
    _use_progress_bar: RwLock<Option<UseProgressBar>>,
    /// Proxy settings
    _proxy_chain: RwLock<ProxyChain>,
    #[cfg(feature = "server")]
    _cors_entries: RwLock<Vec<CorsEntry>>,
    _fanbox_http_headers: RwLock<HeaderMap>,
}

impl OptHelper {
    /// Whether to add artworks to pixiv's history. Only works for APP API.
    pub fn add_history(&self) -> bool {
        if self.opt.get_ref().add_history.is_some() {
            return self.opt.get_ref().add_history.unwrap();
        }
        if self.settings.get_ref().have_bool("add-history") {
            return self.settings.get_ref().get_bool("add-history").unwrap();
        }
        false
    }

    /// return author name filters
    pub fn author_name_filters<'a>(&'a self) -> Option<RwLockReadGuard<'a, Vec<AuthorNameFilter>>> {
        if self.settings.get_ref().have("author-name-filters") {
            return Some(self._author_name_filters.get_ref());
        }
        None
    }

    /// return cookies location, no any check
    pub fn cookies(&self) -> Option<String> {
        if self.opt.get_ref().cookies.is_some() {
            self.opt.get_ref().cookies.clone()
        } else if self.settings.get_ref().have_str("cookies") {
            self.settings.get_ref().get_str("cookies")
        } else {
            None
        }
    }

    #[cfg(feature = "server")]
    #[inline]
    pub fn cors_entries(&self) -> Vec<CorsEntry> {
        self._cors_entries.get_ref().clone()
    }

    #[cfg(feature = "server")]
    pub fn cors_allow_all(&self) -> bool {
        if self.settings.get_ref().have_bool("cors-allow-all") {
            return self
                .settings
                .get_ref()
                .get_bool("cors-allow-all")
                .unwrap_or(false);
        }
        false
    }

    #[cfg(feature = "db")]
    /// Return the config of the database
    pub fn db(&self) -> PixivDownloaderDbConfig {
        if self.settings.get_ref().have("db") {
            PixivDownloaderDbConfig::new(&self.settings.get_ref().get("db").unwrap()).unwrap()
        } else {
            PixivDownloaderDbConfig::default()
        }
    }

    /// The base directory to save downloaded files
    pub fn download_base(&self) -> String {
        match self.opt.get_ref().download_base {
            Some(ref r) => {
                return r.clone();
            }
            None => {}
        }
        match self.settings.get_ref().get_str("download-base") {
            Some(r) => {
                return r;
            }
            None => {}
        }
        #[cfg(feature = "docker")]
        return String::from("/app/downloads");
        #[cfg(not(feature = "docker"))]
        return String::from("./");
    }

    /// Whether to download multiple posts/artworks at the same time.
    pub fn download_multiple_posts(&self) -> bool {
        match self.opt.get_ref().download_multiple_posts {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have_bool("download-multiple-posts") {
            return self
                .settings
                .get_ref()
                .get_bool("download-multiple-posts")
                .unwrap();
        }
        false
    }

    /// Return max retry count of each part when downloading in multiple thread mode.
    pub fn download_part_retry(&self) -> Option<i64> {
        if self.opt.get_ref().download_part_retry.is_some() {
            return Some(self.opt.get_ref().download_part_retry.unwrap());
        }
        let re = self.settings.get_ref().get("download-part-retry");
        if re.is_some() {
            return Some(re.unwrap().as_i64().unwrap());
        }
        None
    }

    /// Return retry count when downloading failed.
    pub fn download_retry(&self) -> Option<i64> {
        if self.opt.get_ref().download_retry.is_some() {
            return Some(self.opt.get_ref().download_retry.unwrap());
        }
        let re = self.settings.get_ref().get("download-retry");
        if re.is_some() {
            return Some(re.unwrap().as_i64().unwrap());
        }
        self.retry()
    }

    /// Return retry interval when downloading files.
    pub fn download_retry_interval(&self) -> NonTailList<Duration> {
        if self.opt.get_ref().download_retry_interval.is_some() {
            return self
                .opt
                .get_ref()
                .download_retry_interval
                .as_ref()
                .unwrap()
                .clone();
        }
        if self.settings.get_ref().have("download-retry-interval") {
            let v = self
                .settings
                .get_ref()
                .get("download-retry-interval")
                .unwrap();
            return parse_retry_interval_from_json(v).unwrap();
        }
        self.retry_interval()
    }

    /// Whether to enable multiple progress bars.
    pub fn enable_multi_progress_bar(&self) -> bool {
        self.download_multiple_files() && self.max_download_tasks() > 1
    }

    pub fn fanbox_http_headers(&self) -> HeaderMap {
        self._fanbox_http_headers.get_ref().clone()
    }

    /// Return whether to force yuv420p as output pixel format when converting ugoira(GIF) to video.
    pub fn force_yuv420p(&self) -> bool {
        match self.opt.get_ref().force_yuv420p {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have_bool("force-yuv420p") {
            return self.settings.get_ref().get_bool("force-yuv420p").unwrap();
        }
        false
    }

    /// return language
    pub fn language(&self) -> Option<String> {
        if self.opt.get_ref().language.is_some() {
            self.opt.get_ref().language.clone()
        } else if self.settings.get_ref().have_str("language") {
            self.settings.get_ref().get_str("language")
        } else {
            None
        }
    }

    /// Return the maximum number of tasks to download posts/artworks at the same time.
    pub fn max_download_post_tasks(&self) -> usize {
        match self.opt.get_ref().max_download_post_tasks {
            Some(r) => {
                return r;
            }
            None => {}
        }
        match self.settings.get_ref().get("max-download-post-tasks") {
            Some(re) => {
                return re.as_usize().unwrap();
            }
            None => {}
        }
        3
    }

    /// Return the maximum number of tasks to download files at the same time.
    pub fn max_download_tasks(&self) -> usize {
        match self.opt.get_ref().max_download_tasks {
            Some(r) => {
                return r;
            }
            None => {}
        }
        match self.settings.get_ref().get("max-download-tasks") {
            Some(re) => {
                return re.as_usize().unwrap();
            }
            None => {}
        }
        5
    }

    /// Return the maximum threads when downloading file.
    pub fn max_threads(&self) -> u64 {
        match self.opt.get_ref().max_threads {
            Some(r) => {
                return r;
            }
            None => {}
        }
        let re = self.settings.get_ref().get("max-threads");
        if re.is_some() {
            return re.unwrap().as_u64().unwrap();
        }
        8
    }

    /// Return whether to enable multiple threads download.
    pub fn multiple_threads_download(&self) -> bool {
        match self.opt.get_ref().multiple_threads_download {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self
            .settings
            .get_ref()
            .have_bool("multiple-threads-download")
        {
            return self
                .settings
                .get_ref()
                .get_bool("multiple-threads-download")
                .unwrap();
        }
        false
    }

    /// Return the size of the each part when downloading file.
    pub fn part_size(&self) -> u32 {
        match self.opt.get_ref().part_size {
            Some(r) => {
                return r;
            }
            None => {}
        }
        let re = self.settings.get_ref().get("part-size");
        if re.is_some() {
            return parse_u32_size(re.as_ref().unwrap()).unwrap();
        }
        65536
    }

    pub fn refresh_token(&self) -> Option<String> {
        if self.opt.get_ref().refresh_token.is_some() {
            self.opt.get_ref().refresh_token.clone()
        } else if self.settings.get_ref().have_str("refresh-token") {
            self.settings.get_ref().get_str("refresh-token")
        } else {
            None
        }
    }

    pub fn update(&self, opt: CommandOpts, settings: SettingStore) {
        if settings.have("author-name-filters") {
            self._author_name_filters.replace_with2(
                AuthorNameFilter::from_json(settings.get("author-name-filters").unwrap()).unwrap(),
            );
        }
        self._use_progress_bar
            .replace_with2(if opt.use_progress_bar.is_some() {
                Some(UseProgressBar::from(opt.use_progress_bar.unwrap()))
            } else if settings.have("use-progress-bar") {
                Some(UseProgressBar::from(
                    UseOrNot::from_json(settings.get("use-progress-bar").unwrap()).unwrap(),
                ))
            } else {
                None
            });
        if settings.have("proxy") {
            self._proxy_chain
                .replace_with2(ProxyChain::try_from(settings.get("proxy").unwrap()).unwrap());
        }
        #[cfg(feature = "server")]
        if settings.have("cors-entries") {
            self._cors_entries
                .replace_with2(parse_cors_entries(&settings.get("cors-entries").unwrap()).unwrap());
        }
        if settings.have("fanbox-http-headers") {
            self._fanbox_http_headers.replace_with2(
                HeaderMap::from_json(&settings.get("fanbox-http-headers").unwrap()).unwrap(),
            );
        }
        self.opt.replace_with2(opt);
        self.settings.replace_with2(settings);
    }

    /// Whether to use Pixiv APP API first.
    pub fn use_app_api(&self) -> bool {
        if self.opt.get_ref().use_app_api.is_some() {
            return self.opt.get_ref().use_app_api.unwrap();
        }
        if self.settings.get_ref().have("use-app-api") {
            return self.settings.get_ref().get_bool("use-app-api").unwrap();
        }
        false
    }

    pub fn overwrite(&self) -> Option<bool> {
        self.opt.get_ref().overwrite
    }

    /// The proxy chain
    pub fn proxy_chain(&self) -> ProxyChain {
        self._proxy_chain.get_ref().deref().clone()
    }

    pub fn init_log(&self) {
        if self.opt.get_ref().verbose {
            crate::log_cfg::init_with_level(log::LevelFilter::Debug);
        } else {
            match self.log_cfg() {
                Some(cfg) => {
                    crate::log_cfg::init_with_file(cfg);
                }
                None => {}
            }
        }
    }

    /// Return retry count
    pub fn retry(&self) -> Option<i64> {
        if self.opt.get_ref().retry.is_some() {
            return Some(self.opt.get_ref().retry.unwrap());
        }
        let re = self.settings.get_ref().get("retry");
        if re.is_some() {
            return Some(re.unwrap().as_i64().unwrap());
        }
        None
    }

    /// Return retry interval
    pub fn retry_interval(&self) -> NonTailList<Duration> {
        if self.opt.get_ref().retry_interval.is_some() {
            return self.opt.get_ref().retry_interval.as_ref().unwrap().clone();
        }
        if self.settings.get_ref().have("retry-interval") {
            let v = self.settings.get_ref().get("retry-interval").unwrap();
            return parse_retry_interval_from_json(v).unwrap();
        }
        self.default_retry_interval.clone()
    }

    #[cfg(feature = "server")]
    /// Return the server
    pub fn server(&self) -> SocketAddr {
        #[cfg(feature = "docker")]
        return SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
        #[cfg(not(feature = "docker"))]
        {
            match self.opt.get_ref().server {
                Some(server) => {
                    return server;
                }
                None => {}
            }
            if self.settings.get_ref().have("server") {
                let v = self.settings.get_ref().get("server").unwrap();
                return SocketAddr::from_str(v.as_str().unwrap()).unwrap();
            }
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        }
    }

    /// Return whether to use description from Web API when description from APP API is empty.
    pub fn use_web_description(&self) -> bool {
        match self.opt.get_ref().use_web_description {
            Some(t) => {
                return t;
            }
            None => {}
        }
        if self.settings.get_ref().have_bool("use-web-description") {
            return self
                .settings
                .get_ref()
                .get_bool("use-web-description")
                .unwrap();
        }
        true
    }

    /// Return whether to use data from webpage first.
    pub fn use_webpage(&self) -> bool {
        if self.opt.get_ref().use_webpage {
            return true;
        }
        if self.settings.get_ref().have_bool("use-webpage") {
            return self.settings.get_ref().get_bool("use-webpage").unwrap();
        }
        false
    }

    pub fn user_agent(&self) -> String {
        match self.opt.get_ref().user_agent.as_ref() {
            Some(ua) => return ua.to_owned(),
            None => {}
        }
        match self.settings.get_ref().get("user-agent") {
            Some(ua) => return ua.as_str().unwrap().to_owned(),
            None => {}
        }
        String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36")
    }

    #[cfg(feature = "exif")]
    /// Return whether to add/update exif information to image files even
    /// when overwrite are disabled.
    pub fn update_exif(&self) -> bool {
        if self.opt.get_ref().update_exif {
            return true;
        }
        if self.settings.get_ref().have_bool("update-exif") {
            return self.settings.get_ref().get_bool("update-exif").unwrap();
        }
        false
    }

    /// Return whether to use progress bar.
    pub fn use_progress_bar(&self) -> bool {
        if self._use_progress_bar.get_ref().is_some() {
            return self._use_progress_bar.get_ref().as_ref().unwrap().to_bool();
        }
        std::io::stdout().is_terminal()
    }

    /// Return progress bar's template
    pub fn progress_bar_template(&self) -> String {
        if self.settings.get_ref().have("progress-bar-template") {
            return self
                .settings
                .get_ref()
                .get_str("progress-bar-template")
                .unwrap();
        }
        String::from("[{elapsed_precise}] [{wide_bar:.green/yellow}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg:40}")
    }

    /// Return whether to download multiple files at the same time.
    pub fn download_multiple_files(&self) -> bool {
        match self.opt.get_ref().download_multiple_files {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have_bool("download-multiple-files") {
            return self
                .settings
                .get_ref()
                .get_bool("download-multiple-files")
                .unwrap();
        }
        false
    }

    /// The max fps when converting ugoira(GIF) to video.
    pub fn ugoira_max_fps(&self) -> f32 {
        match self.opt.get_ref().ugoira_max_fps {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have("ugoira-max-fps") {
            let v = self.settings.get_ref().get("ugoira-max-fps").unwrap();
            return v.as_f32().unwrap();
        }
        60f32
    }

    /// The Constant Rate Factor when converting ugoira(GIF) to video.
    pub fn x264_crf(&self) -> Option<f32> {
        match self.opt.get_ref().x264_crf {
            Some(r) => {
                return Some(r);
            }
            None => {}
        }
        if self.settings.get_ref().have("x264-crf") {
            let v = self.settings.get_ref().get("x264-crf").unwrap();
            return v.as_f32();
        }
        None
    }

    /// Return the x264 profile when converting ugoira(GIF) to video.
    pub fn x264_profile(&self) -> X264Profile {
        match self.opt.get_ref().x264_profile {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have("x264-profile") {
            let v = self.settings.get_ref().get("x264-profile").unwrap();
            return X264Profile::from_str(v.as_str().unwrap()).unwrap();
        }
        X264Profile::default()
    }

    /// Use page number for pictures' file name in fanbox.
    pub fn fanbox_page_number(&self) -> bool {
        match self.opt.get_ref().fanbox_page_number {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have_bool("fanbox-page-number") {
            return self
                .settings
                .get_ref()
                .get_bool("fanbox-page-number")
                .unwrap();
        }
        false
    }

    #[cfg(feature = "server")]
    /// The server's host name. Used in some proxy.
    pub fn server_base(&self) -> Option<String> {
        self.settings.get_ref().get_str("server-base")
    }

    #[cfg(feature = "server")]
    /// The maximum number of push tasks running at the same time.
    pub fn push_task_max_count(&self) -> usize {
        match self.opt.get_ref().push_task_max_count {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have("push-task-max-count") {
            let v = self.settings.get_ref().get("push-task-max-count").unwrap();
            return v.as_usize().unwrap();
        }
        4
    }

    #[cfg(feature = "server")]
    /// The maximum number of tasks to push to client at the same time.
    pub fn push_task_max_push_count(&self) -> usize {
        match self.opt.get_ref().push_task_max_push_count {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self.settings.get_ref().have("push-task-max-push-count") {
            let v = self
                .settings
                .get_ref()
                .get("push-task-max-push-count")
                .unwrap();
            return v.as_usize().unwrap();
        }
        4
    }

    #[cfg(feature = "server")]
    /// Whether to prevent to run push task.
    pub fn disable_push_task(&self) -> bool {
        self.opt.get_ref().disable_push_task
    }

    #[cfg(feature = "server")]
    pub fn temp_dir(&self) -> PathBuf {
        #[cfg(feature = "docker")]
        return PathBuf::from("/app/temp");
        std::env::temp_dir()
    }

    /// The path to config file of [log4rs]
    pub fn log_cfg(&self) -> Option<String> {
        return self.settings.get_ref().get_str("log-cfg");
    }

    /// The path to ugoira cli executable.
    pub fn ugoira(&self) -> Option<String> {
        match self.opt.get_ref().ugoira.as_ref() {
            Some(d) => {
                return Some(d.clone());
            }
            None => {}
        }
        match self.settings.get_ref().get_str("ugoira") {
            Some(s) => Some(s),
            None => {
                #[cfg(all(feature = "ugoira", any(feature = "docker", windows)))]
                return Some(String::from("ugoira"));
                #[cfg(all(feature = "ugoira", not(any(feature = "docker", windows))))]
                return Some(
                    crate::utils::get_exe_path_else_current()
                        .join("ugoira")
                        .to_string_lossy()
                        .into_owned(),
                );
                #[cfg(not(feature = "ugoira"))]
                return None;
            }
        }
    }

    #[cfg(feature = "ugoira")]
    /// Whether to use ugoira cli.
    pub fn ugoira_cli(&self) -> bool {
        match self.opt.get_ref().ugoira_cli.as_ref() {
            Some(d) => {
                return d.clone();
            }
            None => {}
        }
        match self.settings.get_ref().get_bool("ugoira-cli") {
            Some(d) => d,
            None => false,
        }
    }
}

impl Default for OptHelper {
    fn default() -> Self {
        let mut l = NonTailList::default();
        l += Duration::new(3, 0);
        let s = Self {
            opt: RwLock::new(CommandOpts::default()),
            settings: RwLock::new(SettingStore::default()),
            default_retry_interval: l,
            _author_name_filters: RwLock::new(Vec::new()),
            _use_progress_bar: RwLock::new(None),
            _proxy_chain: RwLock::new(ProxyChain::default()),
            #[cfg(feature = "server")]
            _cors_entries: RwLock::new(Vec::new()),
            _fanbox_http_headers: RwLock::new(HeaderMap::new()),
        };
        s.init_log();
        s
    }
}

lazy_static! {
    #[doc(hidden)]
    pub static ref HELPER: Arc<OptHelper> = Arc::new(OptHelper::default());
}

/// Get a [OptHelper] interface.
pub fn get_helper() -> Arc<OptHelper> {
    Arc::clone(&HELPER)
}
