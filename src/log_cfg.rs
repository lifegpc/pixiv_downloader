use crate::ext::rw_lock::GetRwLock;
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{init_config, Config, Handle};
use std::sync::RwLock;

lazy_static! {
    static ref HANDLE: RwLock<Option<Handle>> = RwLock::new(None);
}

pub fn init_with_level(level: LevelFilter) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();
    let mut h = HANDLE.get_mut();
    if let Some(h) = h.as_ref() {
        h.set_config(config);
    } else {
        let handle = init_config(config).unwrap();
        h.replace(handle);
    }
}

pub fn init_default() {
    init_with_level(LevelFilter::Warn);
}
