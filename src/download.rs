use crate::gettext;
use crate::pixiv::PixivWebClient;
use crate::Main;

impl Main {
    pub fn download(&mut self) -> i32 {
        let mut pw = PixivWebClient::new(&self);
        if !pw.init() {
            println!("{}", gettext("Failed to initialize pixiv web api client."));
            return 1;
        }
        pw.check_login();
        0
    }
}
