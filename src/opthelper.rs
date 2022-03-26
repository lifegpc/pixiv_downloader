use crate::author_name_filter::AuthorNameFilter;
use crate::ext::json::FromJson;
use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;
use crate::opt::use_progress_bar::UseProgressBar;
use crate::opts::CommandOpts;
use crate::list::NonTailList;
use crate::retry_interval::parse_retry_interval_from_json;
use crate::settings::SettingStore;
use std::time::Duration;

/// An sturct to access all available settings/command line switches
#[derive(Clone, Debug)]
pub struct OptHelper<'a> {
    /// Command Line Options
    opt: &'a CommandOpts,
    /// Settings
    settings: &'a SettingStore,
    default_retry_interval: NonTailList<Duration>,
    _author_name_filters: Option<Vec<AuthorNameFilter>>,
    _use_progress_bar: Option<UseProgressBar>,
}

impl<'a> OptHelper<'a> {
    pub fn author_name_filters(&self) -> Option<&Vec<AuthorNameFilter>> {
        if self.settings.have("author-name-filters") {
            return self._author_name_filters.as_ref();
        }
        None
    }

    /// return cookies location, no any check
    pub fn cookies(&self) -> Option<String> {
        if self.opt.cookies.is_some() {
            self.opt.cookies.clone()
        } else if self.settings.have_str("cookies") {
            self.settings.get_str("cookies")
        } else {
            None
        }
    }

    /// return language
    pub fn language(&self) -> Option<String> {
        if self.opt.language.is_some() {
            self.opt.language.clone()
        } else if self.settings.have_str("language") {
            self.settings.get_str("language")
        } else {
            None
        }
    }

    pub fn new(opt: &'a CommandOpts, settings: &'a SettingStore) -> Self {
        let mut l = NonTailList::default();
        l += Duration::new(3, 0);
        let _author_name_filters = if settings.have("author-name-filters") {
            Some(AuthorNameFilter::from_json(settings.get("author-name-filters").unwrap()).unwrap())
        } else {
            None
        };
        let _use_progress_bar = if opt.use_progress_bar.is_some() {
            Some(UseProgressBar::from(opt.use_progress_bar.unwrap()))
        } else if settings.have("use-progress-bar") {
            Some(UseProgressBar::from(UseOrNot::from_json(settings.get("use-progress-bar").unwrap()).unwrap()))
        } else {
            None
        };
        Self {
            opt,
            settings,
            default_retry_interval: l,
            _author_name_filters: _author_name_filters,
            _use_progress_bar: _use_progress_bar,
        }
    }

    pub fn overwrite(&self) -> Option<bool> {
        self.opt.overwrite
    }

    pub fn verbose(&self) -> bool {
        self.opt.verbose
    }

    /// Return retry count
    pub fn retry(&self) -> Option<u64> {
        if self.opt.retry.is_some() {
            return Some(self.opt.retry.unwrap());
        }
        let re = self.settings.get("retry");
        if re.is_some() {
            return Some(re.unwrap().as_u64().unwrap());
        }
        None
    }

    /// Return retry interval
    pub fn retry_interval(&self) -> NonTailList<Duration> {
        if self.opt.retry_interval.is_some() {
            return self.opt.retry_interval.as_ref().unwrap().clone();
        }
        if self.settings.have("retry-interval") {
            let v = self.settings.get("retry-interval").unwrap();
            return parse_retry_interval_from_json(v).unwrap();
        }
        self.default_retry_interval.clone()
    }

    /// Return whether to use data from webpage first.
    pub fn use_webpage(&self) -> bool {
        if self.opt.use_webpage {
            return true;
        }
        if self.settings.have_bool("use-webpage") {
            return self.settings.get_bool("use-webpage").unwrap();
        }
        false
    }

    #[cfg(feature = "exif")]
    /// Return whether to add/update exif information to image files even
    /// when overwrite are disabled.
    pub fn update_exif(&self) -> bool {
        if self.opt.update_exif {
            return true;
        }
        if self.settings.have_bool("update-exif") {
            return self.settings.get_bool("update-exif").unwrap();
        }
        false
    }

    /// Return whether to use progress bar.
    pub fn use_progress_bar(&self) -> bool {
        if self._use_progress_bar.is_some() {
            return self._use_progress_bar.as_ref().unwrap().to_bool();
        }
        atty::is(atty::Stream::Stdout)
    }

    /// Return progress bar's template
    pub fn progress_bar_template(&self) -> String {
        if self.settings.have("progress-bar-template") {
            return self.settings.get_str("progress-bar-template").unwrap()
        }
        String::from("[{elapsed_precise}] [{wide_bar:.green/yellow}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg:40}")
    }
}
