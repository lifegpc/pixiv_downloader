use crate::opts::CommandOpts;
use crate::settings::SettingStore;

#[derive(Clone)]
pub struct OptHelper<'a> {
    opt: &'a CommandOpts,
    settings: &'a SettingStore,
}

impl<'a> OptHelper<'a> {
    pub fn cookies(&self) -> Option<String> {
        if self.opt.cookies.is_some() {
            self.opt.cookies.clone()
        } else if self.settings.have_str("cookies") {
            self.settings.get_str("cookies")
        } else {
            None
        }
    }

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
}
