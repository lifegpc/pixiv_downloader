extern crate spin_on;

use crate::cookies::Cookie;
use crate::cookies::CookieJar;
use crate::gettext;
use crate::list::NonTailList;
use crate::opthelper::OptHelper;
use futures_util::StreamExt;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use json::JsonValue;
use reqwest::{Client, IntoUrl, RequestBuilder, Response};
use spin_on::spin_on;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::remove_file;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub trait ToHeaders {
    fn to_headers(&self) -> Option<HashMap<String, String>>;
}

impl ToHeaders for Option<HashMap<String, String>> {
    fn to_headers(&self) -> Option<HashMap<String, String>> {
        self.clone()
    }
}

impl ToHeaders for HashMap<String, String> {
    fn to_headers(&self) -> Option<HashMap<String, String>> {
        Some(self.clone())
    }
}

impl ToHeaders for JsonValue {
    fn to_headers(&self) -> Option<HashMap<String, String>> {
        if !self.is_object() {
            return None;
        }
        let mut h = HashMap::new();
        for (k, v) in self.entries() {
            let d = if v.is_string() {
                String::from(v.as_str().unwrap())
            } else {
                v.dump()
            };
            h.insert(String::from(k), d);
        }
        Some(h)
    }
}

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
    pub verbose: bool,
    /// Retry times, 0 means disable
    pub retry: u64,
    /// Retry interval
    pub retry_interval: Option<NonTailList<Duration>>,
}

impl WebClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            headers: HashMap::new(),
            cookies: CookieJar::new(),
            verbose: false,
            retry: 3,
            retry_interval: None,
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
    /// Send GET requests with parameters
    /// * `param` - GET parameters. Should be a JSON object/array. If value in map is not a string, will dump it
    /// # Examples
    /// ```
    /// let client = WebClient::new();
    /// client.verbose = true;
    /// client.get_with_param("https://test.com/a", json::object!{"data": "param1"});
    /// client.get_with_param("https://test.com/a", json::object!{"daa": {"ad": "test"}});
    /// client.get_with_param("https://test.com/a", json::array![["daa", "param1"]]);
    /// ```
    /// It will GET `https://test.com/a?data=param1`, `https://test.com/a?daa=%7B%22ad%22%3A%22test%22%7D`, `https://test.com/a?daa=param1`
    pub fn get_with_param<U: IntoUrl + Clone>(&mut self, url: U, param: JsonValue) -> Option<Response> {
        let u = url.into_url();
        if u.is_err() {
            println!("{} \"{}\"", gettext("Can not parse URL:"), u.unwrap_err());
            return None;
        }
        let mut u = u.unwrap();
        if !param.is_object() && !param.is_array() {
            println!(
                "{} \"{}\"",
                gettext("Parameters should be object or array:"),
                param
            );
            return None;
        }
        {
            let mut query = u.query_pairs_mut();
            if param.is_object() {
                for (k, v) in param.entries() {
                    let s: String;
                    if v.is_string() {
                        s = String::from(v.as_str().unwrap());
                    } else {
                        s = v.dump();
                    }
                    query.append_pair(k, s.as_str());
                }
            } else {
                for v in param.members() {
                    if !v.is_object() {
                        println!("{} \"{}\"", gettext("Parameters should be array:"), v);
                        return None;
                    }
                    if v.len() < 2 {
                        println!("{} \"{}\"", gettext("Parameters need at least a value:"), v);
                        return None;
                    }
                    let okey = &v[0];
                    let key: String;
                    if okey.is_string() {
                        key = String::from(okey.as_str().unwrap());
                    } else {
                        key = okey.dump();
                    }
                    let mut mems = v.members();
                    mems.next();
                    for val in mems {
                        let s: String;
                        if val.is_string() {
                            s = String::from(val.as_str().unwrap());
                        } else {
                            s = val.dump();
                        }
                        query.append_pair(key.as_str(), s.as_str());
                    }
                }
            }
        }
        self.get(u.as_str(), None)
    }

    pub fn get<U: IntoUrl + Clone, H: ToHeaders + Clone>(&mut self, url: U, headers: H) -> Option<Response> {
        let mut count = 0u64;
        while count <= self.retry {
            let r = self._get(url.clone(), headers.clone());
            if r.is_some() {
                return r;
            }
            count += 1;
            if count <= self.retry {
                if self.retry_interval.is_some() {
                    let t = self.retry_interval.as_ref().unwrap()[(count - 1).try_into().unwrap()];
                    if !t.is_zero() {
                        println!("{}", gettext("Retry after <num> seconds.").replace("<num>", format!("{}", t.as_secs_f64()).as_str()).as_str());
                        spin_on(tokio::time::sleep(t));
                    }
                }
                println!("{}", gettext("Retry <count> times now.").replace("<count>", format!("{}", count).as_str()).as_str());
            }
        }
        None
    }

    pub async fn aget<U: IntoUrl + Clone, H: ToHeaders + Clone>(&mut self, url: U, headers: H) -> Option<Response> {
        let mut count = 0u64;
        while count <= self.retry {
            let r = self._aget2(url.clone(), headers.clone()).await;
            if r.is_some() {
                return r;
            }
            count += 1;
            if count <= self.retry {
                let t = self.retry_interval.as_ref().unwrap()[(count - 1).try_into().unwrap()];
                if !t.is_zero() {
                    println!("{}", gettext("Retry after <num> seconds.").replace("<num>", format!("{}", t.as_secs_f64()).as_str()).as_str());
                    tokio::time::sleep(t).await;
                }
            }
            println!("{}", gettext("Retry <count> times now.").replace("<count>", format!("{}", count).as_str()).as_str());
        }
        None
    }

    /// Send GET requests
    pub fn _get<U: IntoUrl, H: ToHeaders>(&mut self, url: U, headers: H) -> Option<Response> {
        let r = self._aget(url, headers);
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
        if self.verbose {
            println!("{}", r.status());
        }
        Some(r)
    }

    pub async fn _aget2<U: IntoUrl, H: ToHeaders>(&mut self, url: U, headers: H) -> Option<Response> {
        let r = self._aget(url, headers);
        let r = r.send().await;
        match r {
            Ok(_) => {}
            Err(e) => {
                println!("{} {}", gettext("Error when request:"), e);
                return None;
            }
        }
        let r = r.unwrap();
        self.handle_set_cookie(&r);
        if self.verbose {
            println!("{}", r.status());
        }
        Some(r)
    }

    pub fn _aget<U: IntoUrl, H: ToHeaders>(&mut self, url: U, headers: H) -> RequestBuilder {
        let s = url.as_str();
        if self.verbose {
            println!("GET {}", s);
        }
        let mut r = self.client.get(s);
        for (k, v) in self.headers.iter() {
            r = r.header(k, v);
        }
        let headers = headers.to_headers();
        if headers.is_some() {
            let h = headers.unwrap();
            for (k, v) in h.iter() {
                r = r.header(k, v);
            }
        }
        let c = gen_cookie_header(&mut self.cookies, s);
        if c.len() > 0 {
            r = r.header("Cookie", c.as_str());
        }
        r
    }

    pub async fn adownload_stream<S: AsRef<OsStr> + ?Sized>(file_name: &S, r: Response, opt: &OptHelper, progress_bars: Option<Arc<MultiProgress>>) -> Result<(), ()> {
        let content_length = r.content_length();
        let use_progress_bar = match &content_length {
            Some(_) => { opt.use_progress_bar() }
            None => { false }
        };
        let mut bar = if use_progress_bar {
            Some(ProgressBar::new(content_length.unwrap()))
        } else {
            None
        };
        let p = Path::new(file_name);
        if bar.is_some() {
            bar.as_mut().unwrap().set_style(ProgressStyle::default_bar()
                .template(opt.progress_bar_template().as_ref()).unwrap()
                .progress_chars("#>-"));
            let tmp = p.file_name().unwrap_or(p.as_os_str());
            bar.as_mut().unwrap().set_message(gettext("Downloading \"<loc>\".").replace("<loc>", tmp.to_str().unwrap_or("<NULL>")));
            if progress_bars.is_some() {
                bar = Some(progress_bars.unwrap().add(bar.unwrap()));
            }
        }
        if p.exists() {
            let re = remove_file(p);
            if re.is_err() {
                if bar.is_none() {
                    println!("{} {}", gettext("Failed to remove file:"), re.unwrap_err());
                } else {
                    bar.as_ref().unwrap().set_message(format!("{} {}", gettext("Failed to remove file:"), re.unwrap_err()));
                    bar.as_ref().unwrap().abandon();
                }
                return Err(());
            }
        }
        let f = File::create(p);
        if f.is_err() {
            if bar.is_none() {
                println!("{} {}", gettext("Failed to create file:"), f.unwrap_err());
            } else {
                bar.as_ref().unwrap().set_message(format!("{} {}", gettext("Failed to create file:"), f.unwrap_err()));
                bar.as_ref().unwrap().abandon();
            }
            return Err(());
        }
        let mut f = f.unwrap();
        let mut stream = r.bytes_stream();
        while let Some(data) = stream.next().await {
            if data.is_err() {
                if bar.is_none() {
                    println!("{} {}", gettext("Error when downloading file:"), data.unwrap_err());
                } else {
                    bar.as_ref().unwrap().set_message(format!("{} {}", gettext("Error when downloading file:"), data.unwrap_err()));
                    bar.as_ref().unwrap().abandon();
                }
                return Err(());
            }
            let data = data.unwrap();
            if bar.is_some() {
                bar.as_ref().unwrap().inc(data.len() as u64);
                bar.as_ref().unwrap().tick();
            }
            let r = f.write(&data);
            if r.is_err() {
                if bar.is_none() {
                    println!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
                } else {
                    bar.as_ref().unwrap().set_message(format!("{} {}", gettext("Failed to write file:"), r.unwrap_err()));
                    bar.as_ref().unwrap().abandon();
                }
                return Err(());
            }
        }
        if bar.is_some() {
            bar.as_mut().unwrap().finish_with_message(format!("{} {}", gettext("Downloaded image:"), p.to_str().unwrap_or("(null)")));
        }
        Ok(())
    }

    /// Download a stream
    /// * `file_name` - File name
    /// * `r` - Response
    /// Note: If file already exists, will remove existing file first.
    pub fn download_stream<S: AsRef<OsStr> + ?Sized>(file_name: &S, r: Response, opt: &OptHelper) -> Result<(), ()> {
        let content_length = r.content_length();
        let use_progress_bar = match &content_length {
            Some(_) => { opt.use_progress_bar() }
            None => { false }
        };
        let mut bar = if use_progress_bar {
            Some(ProgressBar::new(content_length.unwrap()))
        } else {
            None
        };
        if bar.is_some() {
            bar.as_mut().unwrap().set_style(ProgressStyle::default_bar()
                .template(opt.progress_bar_template().as_ref()).unwrap()
                .progress_chars("#>-"));
        }
        let mut downloaded = 0usize;
        let p = Path::new(file_name);
        if p.exists() {
            let re = remove_file(p);
            if re.is_err() {
                println!("{} {}", gettext("Failed to remove file:"), re.unwrap_err());
                return Err(());
            }
        }
        if bar.is_some() {
            let tmp = p.file_name().unwrap_or(p.as_os_str());
            bar.as_mut().unwrap().set_message(gettext("Downloading \"<loc>\".").replace("<loc>", tmp.to_str().unwrap_or("<NULL>")));
        }
        let f = File::create(p);
        if f.is_err() {
            println!("{} {}", gettext("Failed to create file:"), f.unwrap_err());
            return Err(());
        }
        let mut f = f.unwrap();
        let mut stream = r.bytes_stream();
        while let Some(data) = spin_on(stream.next()) {
            if data.is_err() {
                println!("{} {}", gettext("Error when downloading file:"), data.unwrap_err());
                return Err(());
            }
            let data = data.unwrap();
            downloaded += data.len();
            if bar.is_some() {
                bar.as_mut().unwrap().set_position(downloaded as u64);
            }
            let r = f.write(&data);
            if r.is_err() {
                println!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
                return Err(());
            }
        }
        Ok(())
    }
}
