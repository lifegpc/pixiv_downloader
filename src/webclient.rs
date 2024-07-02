use crate::cookies::Cookie;
use crate::cookies::ManagedCookieJar;
use crate::error::PixivDownloaderError;
use crate::ext::atomic::AtomicQuick;
use crate::ext::json::ToJson;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use crate::list::NonTailList;
use crate::opthelper::get_helper;
use json::JsonValue;
use proc_macros::print_error;
use reqwest::{Client, ClientBuilder, IntoUrl, Request, Response};
use serde::ser::Serialize;
use std::collections::HashMap;
use std::default::Default;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::Duration;

/// Convert data to HTTP headers map
pub trait ToHeaders {
    /// return HTTP headers map
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
pub fn gen_cookie_header<U: IntoUrl>(c: &WebClient, url: U) -> String {
    c.get_cookies_as_mut().jar.get_mut().check_expired();
    let mut s = String::from("");
    let u = url.as_str();
    for a in c.get_cookies().jar.get_ref().iter() {
        if a.matched(u) {
            if s.len() > 0 {
                s += " ";
            }
            s += a.get_name_value().as_str();
        }
    }
    s
}

pub trait ReqMiddleware {
    fn handle(&self, r: Request, c: Client) -> Result<Request, PixivDownloaderError>;
}

impl<T> ReqMiddleware for Arc<T>
where
    T: ReqMiddleware,
{
    fn handle(&self, r: Request, c: Client) -> Result<Request, PixivDownloaderError> {
        self.as_ref().handle(r, c)
    }
}

/// A Web Client
pub struct WebClient {
    /// Basic Web Client
    client: Client,
    /// HTTP Headers
    headers: RwLock<HashMap<String, String>>,
    /// Cookies
    cookies: RwLock<ManagedCookieJar>,
    /// Retry times, 0 means disable, < 0 means always retry
    retry: Arc<AtomicI64>,
    /// Retry interval
    retry_interval: RwLock<Option<NonTailList<Duration>>>,
    /// Request middlewares
    req_middlewares: RwLock<Vec<Box<dyn ReqMiddleware + Send + Sync>>>,
}

impl WebClient {
    /// Create a new instance of client
    ///
    /// This function will not handle any basic options, please use [Self::default()] instead.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            headers: RwLock::new(HashMap::new()),
            cookies: RwLock::new(ManagedCookieJar::new()),
            retry: Arc::new(AtomicI64::new(3)),
            retry_interval: RwLock::new(None),
            req_middlewares: RwLock::new(Vec::new()),
        }
    }

    fn handle_req_middlewares(&self, r: Request) -> Result<Request, PixivDownloaderError> {
        let mut r = r;
        for i in self.req_middlewares.get_ref().iter() {
            r = i.handle(r, self.client.clone())?;
        }
        Ok(r)
    }

    pub fn add_req_middleware(&self, m: Box<dyn ReqMiddleware + Send + Sync>) {
        self.req_middlewares.get_mut().push(m);
    }

    pub fn get_cookies_as_mut<'a>(&'a self) -> RwLockWriteGuard<'a, ManagedCookieJar> {
        self.cookies.get_mut()
    }

    pub fn get_cookies<'a>(&'a self) -> RwLockReadGuard<'a, ManagedCookieJar> {
        self.cookies.get_ref()
    }

    pub fn get_headers_as_mut<'a>(&'a self) -> RwLockWriteGuard<'a, HashMap<String, String>> {
        self.headers.get_mut()
    }

    pub fn get_headers<'a>(&'a self) -> RwLockReadGuard<'a, HashMap<String, String>> {
        self.headers.get_ref()
    }

    /// return retry times, 0 means disable
    pub fn get_retry(&self) -> i64 {
        self.retry.qload()
    }

    pub fn get_retry_interval_as_mut<'a>(
        &'a self,
    ) -> RwLockWriteGuard<'a, Option<NonTailList<Duration>>> {
        self.retry_interval.get_mut()
    }

    pub fn get_retry_interval<'a>(&'a self) -> RwLockReadGuard<'a, Option<NonTailList<Duration>>> {
        self.retry_interval.get_ref()
    }

    /// Used to handle Set-Cookie header in an [Response]
    /// * `r` - reference to an [Response]
    pub fn handle_set_cookie(&self, r: &Response) {
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
                            self.get_cookies_as_mut().jar.get_mut().add(c);
                        }
                        None => {
                            log::warn!("{}", gettext("Failed to parse Set-Cookie header."));
                        }
                    }
                }
                Err(e) => {
                    log::warn!("{} {}", gettext("Failed to convert to string:"), e);
                }
            }
        }
    }

    /// Read cookies from file.
    /// * `file_name`: File name
    ///
    /// returns true if readed successfully.
    /// # Note
    /// If read failed, will clean all entries in the current [ManagedCookieJar]
    pub fn read_cookies(&self, file_name: &str) -> bool {
        let mut c = self.get_cookies_as_mut();
        let r = c.read(file_name);
        if !r {
            c.jar.get_mut().clear();
        }
        r
    }

    /// Set new HTTP header
    /// * `key` - The key of the new HTTP header
    /// * `value` - The value of the new HTTP value
    ///
    /// Returns the old HTTP header value if presented.
    pub fn set_header(&self, key: &str, value: &str) -> Option<String> {
        self.get_headers_as_mut()
            .insert(String::from(key), String::from(value))
    }

    /// Set retry times, 0 means disable
    pub fn set_retry(&self, retry: i64) {
        self.retry.qstore(retry)
    }

    /// Send GET requests with parameters
    /// * `param` - GET parameters. Should be a JSON object/array. If value in map is not a string, will dump it
    /// # Examples
    /// ```
    /// let client = WebClient::new();
    /// client.set_verbose(true);
    /// client.get_with_param("https://test.com/a", json::object!{"data": "param1"}, None);
    /// client.get_with_param("https://test.com/a", json::object!{"daa": {"ad": "test"}}, None);
    /// client.get_with_param("https://test.com/a", json::array![["daa", "param1"]], None);
    /// ```
    /// It will GET `https://test.com/a?data=param1`, `https://test.com/a?daa=%7B%22ad%22%3A%22test%22%7D`, `https://test.com/a?daa=param1`
    pub async fn get_with_param<U: IntoUrl + Clone, J: ToJson, H: ToHeaders + Clone>(
        &self,
        url: U,
        param: J,
        headers: H,
    ) -> Option<Response> {
        let u = url.into_url();
        if u.is_err() {
            log::error!("{} \"{}\"", gettext("Can not parse URL:"), u.unwrap_err());
            return None;
        }
        let mut u = u.unwrap();
        let obj = param.to_json();
        if obj.is_none() {
            return self.get(u, headers).await;
        }
        let obj = obj.unwrap();
        if !obj.is_object() && !obj.is_array() {
            log::error!(
                "{} \"{}\"",
                gettext("Parameters should be object or array:"),
                obj
            );
            return None;
        }
        {
            let mut query = u.query_pairs_mut();
            if obj.is_object() {
                for (k, v) in obj.entries() {
                    let s: String;
                    if v.is_string() {
                        s = String::from(v.as_str().unwrap());
                    } else {
                        s = v.dump();
                    }
                    query.append_pair(k, s.as_str());
                }
            } else {
                for v in obj.members() {
                    if !v.is_object() {
                        log::error!("{} \"{}\"", gettext("Parameters should be array:"), v);
                        return None;
                    }
                    if v.len() < 2 {
                        log::error!("{} \"{}\"", gettext("Parameters need at least a value:"), v);
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
        self.get(u.as_str(), headers).await
    }

    /// Send Get Requests
    pub async fn get<U: IntoUrl + Clone, H: ToHeaders + Clone>(
        &self,
        url: U,
        headers: H,
    ) -> Option<Response> {
        let mut count = 0i64;
        let retry = self.get_retry();
        while retry < 0 || count <= retry {
            let r = self._aget2(url.clone(), headers.clone()).await;
            if r.is_some() {
                return r;
            }
            count += 1;
            if retry < 0 || count <= retry {
                let t =
                    self.get_retry_interval().as_ref().unwrap()[(count - 1).try_into().unwrap()];
                if !t.is_zero() {
                    log::info!(
                        "{}",
                        gettext("Retry after <num> seconds.")
                            .replace("<num>", format!("{}", t.as_secs_f64()).as_str())
                            .as_str()
                    );
                    tokio::time::sleep(t).await;
                }
            }
            log::info!(
                "{}",
                gettext("Retry <count> times now.")
                    .replace("<count>", format!("{}", count).as_str())
                    .as_str()
            );
        }
        None
    }

    /// Send GET requests without retry
    pub async fn _aget2<U: IntoUrl, H: ToHeaders>(&self, url: U, headers: H) -> Option<Response> {
        let r = print_error!(
            gettext("Failed to generate request:"),
            self._aget(url, headers)
        );
        let r = print_error!(gettext("Error when request:"), self.client.execute(r).await);
        self.handle_set_cookie(&r);
        log::debug!(target: "webclient", "{}", r.status());
        Some(r)
    }

    /// Generate a requests
    pub fn _aget<U: IntoUrl, H: ToHeaders>(
        &self,
        url: U,
        headers: H,
    ) -> Result<Request, PixivDownloaderError> {
        let s = url.as_str();
        log::debug!(target: "webclient", "GET {}", s);
        let mut r = self.client.get(s);
        for (k, v) in self.get_headers().iter() {
            r = r.header(k, v);
            log::debug!(target: "webclient", "{}: {}", k, v);
        }
        let headers = headers.to_headers();
        if headers.is_some() {
            let h = headers.unwrap();
            for (k, v) in h.iter() {
                r = r.header(k, v);
                log::debug!(target: "webclient", "{}: {}", k, v);
            }
        }
        let c = gen_cookie_header(&self, s);
        if c.len() > 0 {
            r = r.header("Cookie", c.as_str());
            log::debug!(target: "webclient", "Cookie: {}", c.as_str());
        }
        self.handle_req_middlewares(r.build()?)
    }

    pub async fn post<U: IntoUrl + Clone, H: ToHeaders + Clone, S: Serialize + Clone>(
        &self,
        url: U,
        headers: H,
        form: Option<S>,
    ) -> Option<Response> {
        let mut count = 0i64;
        let retry = self.get_retry();
        while retry < 0 || count <= retry {
            let r = self
                ._apost2(url.clone(), headers.clone(), form.clone())
                .await;
            if r.is_some() {
                return r;
            }
            count += 1;
            if retry < 0 || count <= retry {
                let t =
                    self.get_retry_interval().as_ref().unwrap()[(count - 1).try_into().unwrap()];
                if !t.is_zero() {
                    log::info!(
                        "{}",
                        gettext("Retry after <num> seconds.")
                            .replace("<num>", format!("{}", t.as_secs_f64()).as_str())
                            .as_str()
                    );
                    tokio::time::sleep(t).await;
                }
            }
            log::info!(
                "{}",
                gettext("Retry <count> times now.")
                    .replace("<count>", format!("{}", count).as_str())
                    .as_str()
            );
        }
        None
    }

    pub async fn _apost2<U: IntoUrl, H: ToHeaders, S: Serialize>(
        &self,
        url: U,
        headers: H,
        form: Option<S>,
    ) -> Option<Response> {
        let r = print_error!(
            gettext("Failed to generate request:"),
            self._apost(url, headers, form)
        );
        let r = print_error!(gettext("Error when request:"), self.client.execute(r).await);
        self.handle_set_cookie(&r);
        log::debug!(target: "webclient","{}", r.status());
        Some(r)
    }

    /// Generate a POST request
    pub fn _apost<U: IntoUrl, H: ToHeaders, S: Serialize>(
        &self,
        url: U,
        headers: H,
        form: Option<S>,
    ) -> Result<Request, PixivDownloaderError> {
        let s = url.as_str();
        log::debug!(target: "webclient", "POST {}", s);
        let mut r = self.client.post(s);
        for (k, v) in self.get_headers().iter() {
            r = r.header(k, v);
        }
        let headers = headers.to_headers();
        if headers.is_some() {
            let h = headers.unwrap();
            for (k, v) in h.iter() {
                r = r.header(k, v);
            }
        }
        let c = gen_cookie_header(&self, s);
        if c.len() > 0 {
            r = r.header("Cookie", c.as_str());
        }
        match form {
            Some(f) => {
                r = r.form(&f);
            }
            None => {}
        }
        self.handle_req_middlewares(r.build()?)
    }
}

impl Default for WebClient {
    fn default() -> Self {
        let opt = get_helper();
        let mut c = ClientBuilder::new();
        let chain = opt.proxy_chain();
        if !chain.is_empty() {
            c = c.proxy(reqwest::Proxy::custom(move |url| chain.r#match(url)));
        }
        let c = c.build().unwrap();
        let c = Self::new(c);
        match opt.retry() {
            Some(retry) => c.set_retry(retry),
            None => {}
        }
        c.get_retry_interval_as_mut().replace(opt.retry_interval());
        c
    }
}
