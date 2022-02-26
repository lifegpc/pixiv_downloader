extern crate spin_on;

use crate::cookies::Cookie;
use crate::cookies::CookieJar;
use crate::gettext;
use reqwest::{Client, IntoUrl, RequestBuilder, Response};
use std::collections::HashMap;
use spin_on::spin_on;

/// Generate `cookie` header for a url
/// * `c` - Cookies
/// * `url` - URL
pub fn gen_cookie_header<U: IntoUrl>(c: &mut CookieJar, url: U) -> String {
    c.check_expired();
    let mut s = String::from("");
    let u = url.as_str();
    for a in c.iter() {
        if a.matched(u) {
            if s.len() > 0 {
                s += " ";
            }
            s += a.get_name_value().as_str();
        }
    }
    s
}

/// A Web Client
pub struct WebClient {
    client: Client,
    /// HTTP Headers
    pub headers: HashMap<String, String>,
    cookies: CookieJar,
}

impl WebClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            headers: HashMap::new(),
            cookies: CookieJar::new(),
        }
    }

    pub fn handle_set_cookie(&mut self, r: &Response) {
        let u = r.url();
        let h = r.headers();
        let v = h.get_all("Set-Cookie");
        for val in v {
            let val = val.to_str();
            match val {
                Ok(val) => {
                    let c = Cookie::from_set_cookie(u.as_str(), val);
                    match c {
                        Some(c) => {
                            self.cookies.add(c);
                        }
                        None => {
                            println!("{}", gettext("Failed to parse Set-Cookie header."));
                        }
                    }
                }
                Err(e) => {
                    println!("{} {}", gettext("Failed to convert to string:"), e);
                }
            }
        }
    }

    pub fn read_cookies(&mut self, file_name: &str) -> bool {
        let r = self.cookies.read(file_name);
        if !r {
            self.cookies = CookieJar::new();
        }
        r
    }

    pub fn save_cookies(&mut self, file_name: &str) -> bool {
        self.cookies.save(file_name)
    }

    pub fn set_header(&mut self, key: &str, value: &str) -> Option<String> {
        self.headers.insert(String::from(key), String::from(value))
    }

    /// Send GET requests
    pub fn get<U: IntoUrl>(&mut self, url: U) -> Option<Response> {
        let r = self.aget(url);
        let r = r.send();
        let r = spin_on(r);
        match r {
            Ok(_) => {}
            Err(e) => {
                println!("{} {}", gettext("Error when request:"), e);
                return None;
            }
        }
        let r = r.unwrap();
        self.handle_set_cookie(&r);
        Some(r)
    }

    pub fn aget<U: IntoUrl>(&mut self, url: U) -> RequestBuilder {
        let s = url.as_str();
        let mut r = self.client.get(s);
        for (k, v) in self.headers.iter() {
            r = r.header(k, v);
        }
        let c = gen_cookie_header(&mut self.cookies, s);
        if c.len() > 0 {
            r = r.header("Cookie", c.as_str());
        }
        r
    }
}
