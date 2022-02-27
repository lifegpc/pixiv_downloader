use crate::opts::CommandOpts;
use crate::settings::SettingStore;

/// An sturct to access all available settings/command line switches
#[derive(Clone)]
pub struct OptHelper<'a> {
    /// Command Line Options
    opt: &'a CommandOpts,
    /// Settings
    settings: &'a SettingStore,
}

impl<'a> OptHelper<'a> {
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
        Self { opt, settings }
    }

    pub fn overwrite(&self) -> Option<bool> {
        self.opt.overwrite
    }

    pub fn verbose(&self) -> bool {
        self.opt.verbose
    }
}
