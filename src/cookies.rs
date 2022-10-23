use crate::ext::replace::ReplaceWith;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use reqwest::IntoUrl;
use std::collections::HashMap;
#[cfg(test)]
use std::fs::create_dir;
use std::fs::{remove_file, File};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

trait ToNetscapeStr {
    fn to_netscape_str(&self) -> &'static str;
}

impl ToNetscapeStr for bool {
    fn to_netscape_str(&self) -> &'static str {
        if *self {
            "TRUE"
        } else {
            "FALSE"
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Cookies structure
pub struct Cookie {
    /// Cookie's name
    _name: String,
    /// Cookie's value
    _value: String,
    /// Whether to include subdomains
    _subdomains: bool,
    /// Cookie's Path
    _path: String,
    /// HTTP only
    _http_only: bool,
    /// Expired time
    _expired: Option<DateTime<Utc>>,
    /// Domain name
    _domain: String,
}

impl Cookie {
    pub fn new(
        name: &str,
        value: &str,
        domain: &str,
        subdomains: bool,
        path: &str,
        http_only: bool,
        expired: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            _name: name.to_string(),
            _value: value.to_string(),
            _subdomains: subdomains,
            _path: path.to_string(),
            _http_only: http_only,
            _expired: expired.clone(),
            _domain: domain.to_string(),
        }
    }

    pub fn from_set_cookie<U: IntoUrl>(url: U, header: &str) -> Option<Self> {
        let mut subdomain = false;
        let mut http_only = false;
        let mut expired: i64 = 0;
        let u = url.into_url();
        if u.is_err() {
            println!(
                "{} {}",
                gettext("Warning: Failed to parse URL:"),
                u.unwrap_err()
            );
            return None;
        }
        let u = u.unwrap();
        let mut path = u.path().to_string();
        let t = String::from(header);
        let l: Vec<&str> = t.split(";").collect();
        let m = l[0];
        let t = String::from(m);
        let l2: Vec<&str> = t.split("=").collect();
        if l2.len() < 2 {
            println!(
                "{} {}",
                gettext("Warning: Failed to parse cookie's key and value:"),
                m
            );
            return None;
        }
        let key = l2[0];
        let value = l2[1];
        let mut domain = match u.host_str() {
            Some(s) => Some(String::from(s)),
            None => None,
        };
        for v in l.iter().skip(2) {
            let t = String::from(*v).trim().to_string();
            let ll: Vec<&str> = t.split("=").collect();
            let k = ll[0].to_lowercase();
            if k == "expires" {
                if ll.len() < 2 {
                    println!("{}", gettext("Warning: Expires need a date."));
                    return None;
                }
                let mut re = dateparser::parse(ll[1]);
                if re.is_err() {
                    let s = ll[1].replace("-", " ");
                    re = dateparser::parse(s.as_str());
                    if re.is_err() {
                        println!(
                            "{} {}",
                            gettext("Failed to parse UTC string:"),
                            re.unwrap_err()
                        );
                        return None;
                    }
                }
                let r = re.unwrap();
                expired = r.timestamp();
            } else if k == "max-age" {
                if ll.len() < 2 {
                    println!("{}", gettext("Warning: Max-Age need a duration."));
                    return None;
                }
                let re = ll[1].parse::<i64>();
                if re.is_err() {
                    println!(
                        "{} {}",
                        gettext("Failed to parse Max-Age:"),
                        re.unwrap_err()
                    );
                    return None;
                }
                expired = re.unwrap() + Utc::now().timestamp();
            } else if k == "domain" {
                if ll.len() < 2 {
                    println!("{}", gettext("Warning: Domain need a domain."));
                    return None;
                }
                domain = Some(String::from(ll[1]));
            } else if k == "path" {
                if ll.len() < 2 {
                    println!("{}", gettext("Warning: Path need a path."));
                    return None;
                }
                let p = ll[1].to_string();
                if !p.starts_with("/") {
                    println!(
                        "{} {}",
                        gettext("Warning: path is not starts with \"/\":"),
                        p.as_str()
                    );
                    return None;
                }
                path = p;
            } else if k == "httponly" {
                http_only = true;
            } else if k == "secure" || k == "samesite" {
                continue;
            }
        }
        if domain.is_none() {
            println!("{}", gettext("Warning: Failed to get domain."));
            return None;
        }
        let domain = domain.unwrap();
        if domain.starts_with(".") {
            subdomain = true;
        }
        let expired = if expired == 0 {
            None
        } else {
            Some(Utc.timestamp(expired, 0))
        };
        Some(Self::new(
            key,
            value,
            domain.as_str(),
            subdomain,
            path.as_str(),
            http_only,
            expired,
        ))
    }

    /// Get name and value string: name=value;
    pub fn get_name_value(&self) -> String {
        format!("{}={};", self._name.as_str(), self._value.as_str())
    }

    pub fn is_expired(&self) -> bool {
        if self._expired.is_some() {
            let now = Utc::now();
            if now > self._expired.as_ref().unwrap().clone() {
                return true;
            }
        }
        false
    }

    pub fn is_same_key(&self, other: &Self) -> bool {
        self._name == other._name && self._domain == other._domain
    }

    /// Check if url is matched
    /// * `url` - URL
    pub fn matched<U: IntoUrl>(&self, url: U) -> bool {
        let u = url.into_url();
        if u.is_err() {
            println!(
                "{} {}",
                gettext("Warning: Failed to parse URL:"),
                u.unwrap_err()
            );
            return false;
        }
        if self.is_expired() {
            return false;
        }
        let u = u.unwrap();
        let host = u.host_str();
        if host.is_none() {
            return false;
        }
        let host = host.unwrap();
        let subdomain = self._subdomains || self._domain.starts_with(".");
        let domain = if subdomain && !self._domain.starts_with(".") {
            String::from(".") + &self._domain
        } else {
            self._domain.clone()
        };
        if subdomain && !host.ends_with(&domain) && host != &domain[1..] {
            return false;
        }
        if !subdomain && host != domain {
            return false;
        }
        let path = u.path();
        if !path.starts_with(&self._path) {
            return false;
        }
        true
    }

    pub fn expired_time(&self) -> i64 {
        match &self._expired {
            Some(k) => k.timestamp(),
            None => 0,
        }
    }

    pub fn to_netscape_str(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self._domain,
            self._subdomains.to_netscape_str(),
            self._path,
            self._http_only.to_netscape_str(),
            self.expired_time(),
            self._name,
            self._value
        )
    }
}

#[derive(Clone, Debug)]
/// Cookies Jar
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    pub fn add(&mut self, c: Cookie) {
        let mut i = 0;
        while i < self.cookies.len() {
            let a = &self.cookies[i];
            if a.is_same_key(&c) {
                self.cookies[i] = c;
                return;
            }
            i += 1;
        }
        self.cookies.push(c);
    }

    /// Check and remove all expired cookies
    pub fn check_expired(&mut self) {
        let mut i = 0;
        while i < self.cookies.len() {
            let c = &self.cookies[i];
            if c.is_expired() {
                self.cookies.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    #[allow(dead_code)]
    pub fn get<S: AsRef<str> + ?Sized>(&self, name: &S) -> Option<&Cookie> {
        let name = name.as_ref();
        for i in self.cookies.iter() {
            if i._name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn read<P: AsRef<Path> + ?Sized>(&mut self, file_name: &P) -> bool {
        self.cookies.clear();
        let p = file_name.as_ref();
        if !p.exists() {
            println!("{} {}", gettext("Can not find file:"), p.display());
            return false;
        }
        let re = File::open(p);
        if re.is_err() {
            println!("{} {}", gettext("Can not open file:"), p.display());
            return false;
        }
        let f = re.unwrap();
        let r = BufReader::new(f);
        for line in r.lines() {
            let mut l = line.unwrap();
            l = l.trim().to_string();
            if l.starts_with("#") {
                continue;
            }
            let mut s = l.split('\t');
            if s.clone().count() < 7 {
                println!("{} {}", gettext("Invalid cookie:"), l);
                return false;
            }
            let domain = s.next().unwrap();
            let subdomains = s.next().unwrap() != "FALSE";
            let path = s.next().unwrap();
            let http_only = s.next().unwrap() != "FALSE";
            let expired = s.next().unwrap();
            let name = s.next().unwrap();
            let value = s.next().unwrap();
            let tmp = expired.trim().parse::<i64>();
            if tmp.is_err() {
                println!("{} {}", gettext("Can not parse expired time:"), expired);
                return false;
            }
            let tmp = tmp.unwrap();
            let expired = if tmp == 0 {
                None
            } else {
                Some(Utc.timestamp(tmp, 0))
            };
            let c = Cookie::new(name, value, domain, subdomains, path, http_only, expired);
            self.add(c);
        }
        self.check_expired();
        true
    }

    pub fn save<P: AsRef<Path> + ?Sized>(&mut self, file_name: &P) -> bool {
        let p = file_name.as_ref();
        self.check_expired();
        if p.exists() {
            let re = remove_file(p);
            if re.is_err() {
                println!("{} {}", gettext("Failed to remove file:"), re.unwrap_err());
                return false;
            }
        }
        let re = File::create(p);
        if re.is_err() {
            println!("{} {}", gettext("Failed to create file:"), re.unwrap_err());
            return false;
        }
        let mut f = re.unwrap();
        for c in self.cookies.iter() {
            let r = write!(f, "{}", c.to_netscape_str().as_str());
            if r.is_err() {
                println!("{} {}", gettext("Failed to write file:"), r.unwrap_err());
                return false;
            }
        }
        true
    }

    pub fn iter(&self) -> core::slice::Iter<Cookie> {
        self.cookies.iter()
    }
}

struct CookieJarManager {
    jars: RwLock<HashMap<PathBuf, (Arc<RwLock<CookieJar>>, usize)>>,
}

impl CookieJarManager {
    pub fn new() -> Self {
        Self {
            jars: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_cookie_jar<P: AsRef<Path> + ?Sized>(
        &self,
        path: &P,
    ) -> Result<Arc<RwLock<CookieJar>>, ()> {
        let path = path.as_ref().to_owned();
        let mut jars = self.jars.get_mut();
        match jars.get_mut(&path) {
            Some((jar, count)) => {
                *count += 1;
                Ok(jar.clone())
            }
            None => {
                let mut jar = CookieJar::new();
                if jar.read(&path) {
                    let jar = Arc::new(RwLock::new(jar));
                    jars.insert(path, (jar.clone(), 1));
                    Ok(jar)
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn drop_cookie_jar<P: AsRef<Path> + ?Sized>(&self, path: &P) {
        let path = path.as_ref().to_owned();
        let mut jars = self.jars.get_mut();
        match jars.get_mut(&path) {
            Some((jar, count)) => {
                *count -= 1;
                if *count == 0 {
                    if !jar.get_mut().save(&path) {
                        println!(
                            "{} {}",
                            gettext("Warning: Failed to save cookies file:"),
                            path.display()
                        );
                    }
                    jars.remove(&path);
                }
            }
            None => {}
        }
    }
}

lazy_static! {
    static ref MANAGER: CookieJarManager = CookieJarManager::new();
}

#[derive(Clone, Debug)]
/// A cookie jar that make sure there are one cookie interface for a cookies file
pub struct ManagedCookieJar {
    pub jar: Arc<RwLock<CookieJar>>,
    path: Option<PathBuf>,
}

impl ManagedCookieJar {
    pub fn new() -> Self {
        Self {
            jar: Arc::new(RwLock::new(CookieJar::new())),
            path: None,
        }
    }

    pub fn read<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> bool {
        let path = path.as_ref();
        let path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_owned(),
        };
        match self.path.as_ref() {
            Some(p) => {
                if p == &path {
                    return true;
                }
                MANAGER.drop_cookie_jar(p);
                let jar = match MANAGER.get_cookie_jar(&path) {
                    Ok(jar) => jar,
                    Err(()) => return false,
                };
                self.jar.replace_with(jar);
                self.path.replace(path);
                true
            }
            None => {
                let jar = match MANAGER.get_cookie_jar(&path) {
                    Ok(jar) => jar,
                    Err(()) => return false,
                };
                self.jar.replace_with(jar);
                self.path.replace(path);
                true
            }
        }
    }
}

impl Drop for ManagedCookieJar {
    fn drop(&mut self) {
        if let Some(path) = self.path.as_ref() {
            MANAGER.drop_cookie_jar(path);
        }
    }
}

#[test]
fn test_managed_cookie_jar() {
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    {
        let mut jar = CookieJar::new();
        jar.add(Cookie::new(
            "test",
            "de",
            "example.com",
            true,
            "/",
            false,
            None,
        ));
        jar.save("./test/cookies.txt");
    }
    {
        let mut jar = ManagedCookieJar::new();
        assert!(jar.read("./test/cookies.txt"));
        assert_eq!(jar.jar.get_mut().get("test").unwrap()._value, "de");
        let mut jar2 = ManagedCookieJar::new();
        assert!(jar2.read("./test/cookies.txt"));
        jar2.jar.get_mut().add(Cookie::new(
            "test2",
            "de",
            "example.com",
            true,
            "/",
            false,
            None,
        ));
        assert_eq!(jar.jar.get_mut().get("test2").unwrap()._value, "de");
    }
    {
        let mut jar = CookieJar::new();
        assert!(jar.read("./test/cookies.txt"));
        assert_eq!(jar.get("test").unwrap()._value, "de");
        assert_eq!(jar.get("test2").unwrap()._value, "de");
    }
}
