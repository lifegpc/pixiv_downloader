use crate::author_name_filter::AuthorNameFilter;
use crate::ext::json::FromJson;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;
use crate::opt::use_progress_bar::UseProgressBar;
use crate::opts::CommandOpts;
use crate::list::NonTailList;
use crate::retry_interval::parse_retry_interval_from_json;
use crate::settings::SettingStore;
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

    pub fn update(&self, opt: CommandOpts, settings: SettingStore) {
        if settings.have("author-name-filters") {
            self._author_name_filters.replace_with2(AuthorNameFilter::from_json(settings.get("author-name-filters").unwrap()).unwrap());
        }
        self._use_progress_bar.replace_with2(if opt.use_progress_bar.is_some() {
            Some(UseProgressBar::from(opt.use_progress_bar.unwrap()))
        } else if settings.have("use-progress-bar") {
            Some(UseProgressBar::from(UseOrNot::from_json(settings.get("use-progress-bar").unwrap()).unwrap()))
        } else {
            None
        });
        self.opt.replace_with2(opt);
        self.settings.replace_with2(settings);
    }

    pub fn overwrite(&self) -> Option<bool> {
        self.opt.get_ref().overwrite
    }

    pub fn verbose(&self) -> bool {
        self.opt.get_ref().verbose
    }

    /// Return retry count
    pub fn retry(&self) -> Option<u64> {
        if self.opt.get_ref().retry.is_some() {
            return Some(self.opt.get_ref().retry.unwrap());
        }
        let re = self.settings.get_ref().get("retry");
        if re.is_some() {
            return Some(re.unwrap().as_u64().unwrap());
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
            return self.settings.get_ref().get_str("progress-bar-template").unwrap()
        }
        String::from("[{elapsed_precise}] [{wide_bar:.green/yellow}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg:40}")
    }

    /// Return whether to download multiple images at the same time.
    pub fn download_multiple_images(&self) -> bool {
        match self.opt.get_ref().download_multiple_images {
            Some(r) => { return r; }
            None => {}
        }
        if self.settings.get_ref().have_bool("download-multiple-images") {
            return self.settings.get_ref().get_bool("download-multiple-images").unwrap();
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
        }
    }
}

lazy_static!{
    #[doc(hidden)]
    pub static ref HELPER: Arc<OptHelper> = Arc::new(OptHelper::default());
}

/// Get a [OptHelper] interface.
pub fn get_helper() -> Arc<OptHelper> {
    Arc::clone(&HELPER)
}
