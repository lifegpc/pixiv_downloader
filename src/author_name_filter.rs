use crate::data::json::ToJson;
use crate::gettext;
use crate::stdext::TryErr;
use json::JsonValue;
use regex::Regex;
use std::cmp::PartialEq;
use std::convert::From;
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, derive_more::From, PartialEq)]
pub enum AuthorNameFilterError {
    String(String),
    Regex(regex::Error),
}

impl Display for AuthorNameFilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => { f.write_str(s) }
            Self::Regex(r) => { f.write_fmt(format_args!("{} {}", gettext("Failed to parse regex:"), r)) }
        }
    }
}

impl From<&str> for AuthorNameFilterError {
    fn from(s: &str) -> Self {
       Self::String(String::from(s)) 
    }
}

#[derive(Clone, Debug, derive_more::From)]
pub enum AuthorNameFilter {
    Simple(String),
    Regex(Regex),
}

/// Used to filter the author name
pub trait AuthorFiler {
    /// Used to filter the author name
    fn filter(&self, author: &str) -> String;
}

impl AuthorNameFilter {
    pub fn from_json<T: ToJson>(v: T) -> Result<Vec<Self>, AuthorNameFilterError> {
        let v = v.to_json().try_err(gettext("Failed to get JSON object."))?;
        if !v.is_array() {
            Err(gettext("Unsupported JSON type."))?;
        }
        let mut re = Vec::new();
        for j in v.members() {
            re.push(Self::try_from(j)?);
        }
        Ok(re)
    }
}

impl AuthorFiler for AuthorNameFilter {
    fn filter(&self, author: &str) -> String {
        match self {
            Self::Simple(s) => {
                match author.rfind(s) {
                    Some(i) => { String::from(&author[..i]) }
                    None => { String::from(author) }
                }
            }
            Self::Regex(r) => {
                r.replace_all(author, "").to_owned().to_string()
            }
        }
    }
}

impl<T: AuthorFiler> AuthorFiler for Vec<T> {
    fn filter(&self, author: &str) -> String {
        let ori = String::from(author);
        for i in self {
            let r = i.filter(author);
            if r != ori {
                return r;
            }
        }
        return ori;
    }
}

impl From<&str> for AuthorNameFilter {
    fn from(s: &str) -> Self {
        Self::Simple(String::from(s))
    }
}

impl PartialEq for AuthorNameFilter {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Simple(s) => {
                match other {
                    Self::Regex(_) => { false }
                    Self::Simple(t) => { s == t }
                }
            }
            Self::Regex(r) => {
                match other {
                    Self::Simple(_) => { false }
                    Self::Regex(s) => {
                        r.as_str() == s.as_str()
                    }
                }
            }
        }
    }
}

impl TryFrom<&JsonValue> for AuthorNameFilter {
    type Error = AuthorNameFilterError;
    fn try_from(j: &JsonValue) -> Result<Self, Self::Error> {
        if j.is_string() {
            return Ok(Self::from(j.as_str().unwrap()));
        } else if j.is_object() {
            let t = (&j["type"]).as_str().try_err(gettext("Failed to get filter's type."))?.to_lowercase();
            let rule = (&j["rule"]).as_str().try_err(gettext("Failed to get filter's rule."))?;
            if t == "simple" {
                return Ok(Self::from(rule));
            } else if t == "regex" {
                return Ok(Self::from(Regex::new(rule)?));
            } else {
                Err(format!("{} {}", gettext("Unknown filter's type:"), t.as_str()))?;
            }
        } else {
            Err(gettext("Unsupported JSON type."))?;
        };
        return Err(Self::Error::from(""));
    }
}

pub fn check_author_name_filters(v: &JsonValue) -> bool {
    let r = AuthorNameFilter::from_json(v);
    if r.is_err() {
        println!("{} {}", gettext("Failed parse author name filters:"), r.as_ref().unwrap_err());
    }
    r.is_ok()
}

#[test]
fn test_author_name_filter() {
    assert!(AuthorNameFilter::from("s") == AuthorNameFilter::from("s"));
    assert!(AuthorNameFilter::from(Regex::new("s").unwrap()) == AuthorNameFilter::from(Regex::new("s").unwrap()));
    let l = AuthorNameFilter::from_json(json::array!["🌸"]).unwrap();
    assert_eq!(l, vec![AuthorNameFilter::from("🌸")]);
    assert_eq!(l[0].filter("moco🌸お仕事募集中"), String::from("moco"));
    let r = AuthorNameFilter::from(Regex::new(".?お仕事募集中").unwrap());
    assert_eq!(r.filter("moco🌸お仕事募集中"), String::from("moco"));
    let l = AuthorNameFilter::from_json(json::array![{"type": "simple", "rule": "🌸"}, {"type": "regex", "rule": ".?お仕事募集中"}]).unwrap();
    assert_eq!(l, vec![AuthorNameFilter::from("🌸"), AuthorNameFilter::from(r)]);
    assert_eq!(l.filter("moco<お仕事募集中🌸お仕事募集中"), String::from("moco<お仕事募集中"));
    assert_eq!(l.filter("sss@ss@お仕事募集中"), "sss@ss");
    assert_eq!(l.filter("sss🌸ss🌸お仕事募集中"), "sss🌸ss");
}
