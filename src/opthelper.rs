use crate::ext::json::FromJson;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;
use crate::list::NonTailList;
use crate::opt::author_name_filter::AuthorNameFilter;
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
use std::convert::TryFrom;
#[cfg(feature = "server")]
use std::net::IpAddr;
#[cfg(feature = "server")]
use std::net::Ipv4Addr;
#[cfg(feature = "server")]
use std::net::SocketAddr;
use std::ops::Deref;
#[cfg(feature = "server")]
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
}

impl OptHelper {
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

    /// Return the maximun number of tasks to download simultaneously.
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

    /// Return the maximun threads when downloading file.
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
        self.opt.replace_with2(opt);
        self.settings.replace_with2(settings);
    }

    pub fn overwrite(&self) -> Option<bool> {
        self.opt.get_ref().overwrite
    }

    /// The proxy chain
    pub fn proxy_chain(&self) -> ProxyChain {
        self._proxy_chain.get_ref().deref().clone()
    }

    pub fn verbose(&self) -> bool {
        self.opt.get_ref().verbose
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
        atty::is(atty::Stream::Stdout)
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

    /// Return whether to download multiple images at the same time.
    pub fn download_multiple_images(&self) -> bool {
        match self.opt.get_ref().download_multiple_images {
            Some(r) => {
                return r;
            }
            None => {}
        }
        if self
            .settings
            .get_ref()
            .have_bool("download-multiple-images")
        {
            return self
                .settings
                .get_ref()
                .get_bool("download-multiple-images")
                .unwrap();
        }
        false
    }
}

impl Default for OptHelper {
    fn default() -> Self {
        let mut l = NonTailList::default();
        l += Duration::new(3, 0);
        Self {
            opt: RwLock::new(CommandOpts::default()),
            settings: RwLock::new(SettingStore::default()),
            default_retry_interval: l,
            _author_name_filters: RwLock::new(Vec::new()),
            _use_progress_bar: RwLock::new(None),
            _proxy_chain: RwLock::new(ProxyChain::default()),
            #[cfg(feature = "server")]
            _cors_entries: RwLock::new(Vec::new()),
        }
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
