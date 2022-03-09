use crate::gettext;
use std::cmp::PartialEq;
use std::str::FromStr;
use std::time::Duration;

#[allow(dead_code)]
pub enum DurType {
    Second,
    MilliSecond,
    NanoSecond,
}

#[derive(Clone, PartialEq)]
pub struct Dur {
    num: Option<u64>,
    f: Option<f64>,
}

impl Dur {
    pub fn from_num(num: u64) -> Self {
        Self {
            num: Some(num),
            f: None,
        }
    }

    pub fn from_f(f: f64) -> Self {
        Self {
            num: None,
            f: Some(f),
        }
    }

    pub fn to_duration(&self, t: DurType) -> Duration {
        match t {
            DurType::Second => {
                if self.num.is_some() {
                    Duration::from_secs(self.num.unwrap())
                } else {
                    Duration::from_secs_f64(self.f.unwrap())
                }
            }
            DurType::MilliSecond => {
                if self.num.is_some() {
                    Duration::from_millis(self.num.unwrap())
                } else {
                    Duration::from_secs_f64(self.f.unwrap() / 1000f64)
                }
            }
            DurType::NanoSecond => {
                if self.num.is_some() {
                    Duration::from_nanos(self.num.unwrap())
                } else {
                    Duration::from_secs_f64(self.f.unwrap() / 1_000_000_000f64)
                }
            }
        }
    }
}

impl FromStr for Dur {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let num = s.parse::<u64>();
        if num.is_ok() {
            return Ok(Self::from_num(num.unwrap()));
        }
        let f = s.parse::<f64>();
        if num.is_ok() {
            return Ok(Self::from_f(f.unwrap()));
        }
        Err(gettext("Failed to parse duration from string."))
    }
}

impl PartialEq<u64> for Dur {
    fn eq(&self, other: &u64) -> bool {
        if self.num.is_some() {
            self.num.as_ref().unwrap() == other
        } else {
            false
        }
    }
}

impl PartialEq<f64> for Dur {
    fn eq(&self, other: &f64) -> bool {
        if self.f.is_some() {
            self.f.as_ref().unwrap() == other
        } else {
            false
        }
    }
}
