use crate::_avdict;
use crate::ext::cstr::{ToCStr, ToCStrError};
use crate::ext::flagset::ToFlagSet;
use crate::ext::rawhandle::ToRawHandle;
use crate::gettext;
use std::collections::HashMap;
use std::convert::AsMut;
use std::convert::AsRef;
use std::convert::TryFrom;
use std::default::Default;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt::Display;
use std::iter::Iterator;
use std::ops::Drop;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::str::Utf8Error;

/// Error when operate AVDictionary
#[derive(Debug, derive_more::From, PartialEq)]
pub enum AVDictError {
    /// The normal error message
    String(String),
    /// Failed to use UTF-8 to decode string
    Utf8Error(Utf8Error),
    /// The error code from ffmpeg
    CodeError(AVDictCodeError),
    /// The error occured when convert data to the string in C.
    ToCstr(ToCStrError),
}

impl From<&str> for AVDictError {
    fn from(s: &str) -> Self {
        Self::String(String::from(s))
    }
}

impl From<c_int> for AVDictError {
    fn from(i: c_int) -> Self {
        Self::CodeError(AVDictCodeError::from(i))
    }
}

impl Display for AVDictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_str(s),
            Self::Utf8Error(s) => f.write_fmt(format_args!(
                "{} {}",
                gettext("Failed to decode string with UTF-8:"),
                s
            )),
            Self::CodeError(s) => f.write_fmt(format_args!("{}", s)),
            Self::ToCstr(s) => f.write_fmt(format_args!("{}", s)),
        }
    }
}

/// The error returned from the ffmpeg
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct AVDictCodeError {
    /// Error code
    err: c_int,
}

impl AVDictCodeError {
    /// Convert error code to error message
    pub fn to_str(&self) -> Result<String, AVDictError> {
        let s = unsafe { _avdict::avdict_get_errmsg(self.err) };
        if s.is_null() {
            Err(gettext("Out of memory."))?;
        }
        let ss = unsafe { CStr::from_ptr(s) };
        let ss = ss.to_owned();
        unsafe { _avdict::avdict_mfree(s as *mut std::os::raw::c_void) };
        let re = ss.to_str()?;
        Ok(String::from(re))
    }
}

impl Display for AVDictCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_str() {
            Ok(s) => f.write_str(s.as_str()),
            Err(e) => f.write_fmt(format_args!(
                "{} {}",
                gettext("Failed to get error message:"),
                e
            )),
        }
    }
}

impl From<c_int> for AVDictCodeError {
    fn from(i: c_int) -> Self {
        Self { err: i }
    }
}

flagset::flags! {
    /// The flags of Dictionary
    pub enum AVDictFlags: c_int {
        /// Only get an entry with exact-case key match.
        MatchCase = _avdict::AV_DICT_MATCH_CASE as c_int,
        /// Return first entry in a dictionary whose first part corresponds to the search key,
        /// ignoring the suffix of the found key string.
        IgnoreSuffix = _avdict::AV_DICT_IGNORE_SUFFIX as c_int,
        /// Take ownership of a key that's been allocated
        DontStrdupKey = _avdict::AV_DICT_DONT_STRDUP_KEY as c_int,
        /// Take ownership of a value that's been allocated
        DontStrdupVal = _avdict::AV_DICT_DONT_STRDUP_VAL as c_int,
        /// Don't overwrite existing entries.
        DontOverwrite = _avdict::AV_DICT_DONT_OVERWRITE as c_int,
        /// If the entry already exists, append to it.
        Append = _avdict::AV_DICT_APPEND as c_int,
        /// Allow to store several equal keys in the dictionary.
        Multikey = _avdict::AV_DICT_MULTIKEY as c_int,
    }
}

/// An instance of the AVDictionary
pub struct AVDict {
    pub m: *mut _avdict::AVDict,
}

#[allow(dead_code)]
impl AVDict {
    /// Create a new instance of the AVDictionary
    pub fn new() -> Self {
        Self {
            m: 0 as *mut _avdict::AVDict,
        }
    }

    /// Clone a new instance from current [AVDict].
    /// * `flags` - The flags to use when setting entries
    pub fn copy<T: ToFlagSet<AVDictFlags>>(&self, flags: T) -> Result<Self, AVDictError> {
        if self.m.is_null() {
            return Ok(Self::new());
        }
        let mut m = 0 as *mut _avdict::AVDict;
        let pm: *mut *mut _avdict::AVDict = &mut m;
        let re = unsafe { _avdict::avdict_copy(pm, self.m, flags.to_bits()) };
        if re != 0 {
            Err(re)?;
        }
        Ok(Self { m })
    }

    /// Set all entries in the map to the dictionary
    /// * `maps` - The map which contains entries
    /// * `flags` - The flags to use when setting entries.
    pub fn from_map<K: ToCStr, V: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &mut self,
        maps: &HashMap<K, V>,
        flags: F,
    ) -> Result<(), AVDictError> {
        let flags = flags.to_flag_set();
        for (k, v) in maps {
            self.set(k, v, flags)?;
        }
        Ok(())
    }

    /// Get a dictionary value with matching key.
    /// * `key` - The matching key
    /// * `flags` - The flags to control how entry is retrieved.
    pub fn get<K: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &self,
        key: K,
        flags: F,
    ) -> Result<Option<CString>, AVDictError> {
        if self.m.is_null() {
            return Ok(None);
        }
        let k = key.to_cstr()?;
        let re = unsafe {
            _avdict::avdict_get(
                self.m,
                k.as_ptr(),
                0 as *mut _avdict::AVDictEntry,
                flags.to_bits(),
            )
        };
        if !re.is_null() && unsafe { !(*re).value.is_null() } {
            let s = unsafe { CStr::from_ptr((*re).value) };
            let s = s.to_owned();
            return Ok(Some(s));
        }
        Ok(None)
    }

    pub fn get_all<K: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &self,
        key: K,
        flags: F,
    ) -> Result<Option<Vec<CString>>, AVDictError> {
        if self.m.is_null() {
            return Ok(None);
        }
        let k = key.to_cstr()?;
        let mut re = unsafe {
            _avdict::avdict_get(
                self.m,
                k.as_ptr(),
                0 as *mut _avdict::AVDictEntry,
                flags.to_bits(),
            )
        };
        let mut l = Vec::new();
        while !re.is_null() {
            if unsafe { (*re).value.is_null() } {
                Err(gettext("Failed to get value for entry."))?;
            }
            let s = unsafe { CStr::from_ptr((*re).value) };
            let s = s.to_owned();
            l.push(s);
            re = unsafe { _avdict::avdict_get(self.m, k.as_ptr(), re, flags.to_bits()) };
        }
        if l.len() > 0 {
            return Ok(Some(l));
        }
        Ok(None)
    }

    /// Get dictionary entries as a string.
    ///
    /// Return a string containing dictionary's entries.
    /// * `key_val_sep` - character used to separate key from value
    /// * `pairs_sep` - character used to separate two pairs from each other
    /// # Note
    /// String is escaped with backslashes (`\`).
    /// # Warning
    /// Separators cannot be neither `\` nor `\0`. They also cannot be the same.
    pub fn get_string(&self, key_val_sep: char, pairs_sep: char) -> Result<CString, AVDictError> {
        let mut buf = 0 as *mut c_char;
        let pbuf: *mut *mut c_char = &mut buf;
        let re = unsafe {
            _avdict::avdict_get_string(self.m, pbuf, key_val_sep as c_char, pairs_sep as c_char)
        };
        if re != 0 {
            Err(re)?;
        }
        let s = unsafe { CStr::from_ptr(buf) };
        let s = s.to_owned();
        unsafe { _avdict::avdict_avfree(buf as *mut std::os::raw::c_void) };
        Ok(s)
    }

    pub fn iter<'a>(&'a self) -> AVDictItor<'a> {
        AVDictItor {
            d: self,
            cur: 0 as *mut _avdict::AVDictEntry,
            started: false,
        }
    }

    pub fn len(&self) -> usize {
        if self.m.is_null() {
            0
        } else {
            unsafe { _avdict::avdict_count(self.m) as usize }
        }
    }

    /// Parse the key/value pairs list and add the parsed entries to a dictionary.
    ///
    /// In case of failure, all the successfully set entries are stored.
    /// * `s` - string
    /// * `key_val_sep` - a list of characters used to separate key from value
    /// * `pairs_sep` - a list of characters used to separate two pairs from each other
    /// * `flags` - flags to use when adding to dictionary.
    /// StrdupKey and StrdipValue are ignored since the key/value tokens will always be duplicated.
    pub fn parse_string<K: ToCStr, V: ToCStr, S: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &mut self,
        s: K,
        key_val_sep: V,
        pairs_sep: S,
        flags: F,
    ) -> Result<(), AVDictError> {
        let pm: *mut *mut _avdict::AVDict = &mut self.m;
        let s = s.to_cstr()?;
        let k = key_val_sep.to_cstr()?;
        let p = pairs_sep.to_cstr()?;
        let re = unsafe {
            _avdict::avdict_parse_string(pm, s.as_ptr(), k.as_ptr(), p.as_ptr(), flags.to_bits())
        };
        if re != 0 {
            Err(re)?;
        }
        Ok(())
    }

    pub fn set<K: ToCStr, V: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &mut self,
        key: K,
        value: V,
        flags: F,
    ) -> Result<(), AVDictError> {
        let pm: *mut *mut _avdict::AVDict = &mut self.m;
        let k = key.to_cstr()?;
        let v = value.to_cstr()?;
        let re = unsafe { _avdict::avdict_set(pm, k.as_ptr(), v.as_ptr(), flags.to_bits()) };
        if re != 0 {
            Err(re)?;
        }
        Ok(())
    }

    pub fn set_int<K: ToCStr, F: ToFlagSet<AVDictFlags>>(
        &mut self,
        key: K,
        value: i64,
        flags: F,
    ) -> Result<(), AVDictError> {
        let pm: *mut *mut _avdict::AVDict = &mut self.m;
        let k = key.to_cstr()?;
        let re = unsafe { _avdict::avdict_set_int(pm, k.as_ptr(), value, flags.to_bits()) };
        if re != 0 {
            Err(re)?;
        }
        Ok(())
    }
}

impl AsMut<Self> for AVDict {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl AsRef<Self> for AVDict {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for AVDict {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AVDict {
    fn drop(&mut self) {
        if !self.m.is_null() {
            let ptr: *mut *mut _avdict::AVDict = &mut self.m;
            unsafe { _avdict::avdict_free(ptr) };
            self.m = 0 as *mut _avdict::AVDict;
        }
    }
}

impl<K: ToCStr, V: ToCStr> TryFrom<HashMap<K, V>> for AVDict {
    type Error = AVDictError;
    fn try_from(value: HashMap<K, V>) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl<K: ToCStr, V: ToCStr> TryFrom<&HashMap<K, V>> for AVDict {
    type Error = AVDictError;
    fn try_from(value: &HashMap<K, V>) -> Result<Self, Self::Error> {
        let mut d = Self::new();
        for (k, v) in value {
            d.set(k, v, AVDictFlags::MatchCase | AVDictFlags::Multikey)?;
        }
        Ok(d)
    }
}

impl ToRawHandle<_avdict::AVDict> for AVDict {
    unsafe fn to_raw_handle(&self) -> *mut _avdict::AVDict {
        self.m
    }
}

pub struct AVDictItor<'a> {
    d: &'a AVDict,
    cur: *mut _avdict::AVDictEntry,
    started: bool,
}

lazy_static! {
    #[doc(hidden)]
    static ref NULLSTR: CString = CString::new("").unwrap();
}

impl<'a> Iterator for AVDictItor<'a> {
    type Item = (CString, CString);
    fn next(&mut self) -> Option<Self::Item> {
        if self.started && self.cur.is_null() {
            return None;
        }
        self.started = true;
        let m = unsafe { self.d.to_raw_handle() };
        let re = unsafe {
            _avdict::avdict_get(
                m,
                NULLSTR.as_ptr(),
                self.cur as *const _avdict::AVDictEntry,
                AVDictFlags::IgnoreSuffix.to_bits(),
            )
        };
        self.cur = re;
        if re.is_null() {
            return None;
        }
        let k = unsafe { CStr::from_ptr((*re).key) };
        let k = k.to_owned();
        let v = unsafe { CStr::from_ptr((*re).value) };
        let v = v.to_owned();
        Some((k, v))
    }
}

#[test]
fn test_avdict() {
    let e = AVDictCodeError::from(-1);
    assert_eq!(Ok(String::from("Operation not permitted")), e.to_str());
    let mut d = AVDict::new();
    assert_eq!(0, d.len());
    let d2 = d.copy(None).unwrap();
    assert_eq!(0, d2.len());
    d.set("a", String::from("s"), None).unwrap();
    assert_eq!(1, d.len());
    assert_eq!(0, d2.len());
    let mut d2 = d.copy(None).unwrap();
    assert_eq!(1, d2.len());
    assert_eq!(Ok(None), d.get("f", None));
    assert_eq!(Ok(Some(CString::new("s").unwrap())), d.get("a", None));
    d2.set("b", "ok", AVDictFlags::Multikey).unwrap();
    d2.set("b", "test", AVDictFlags::Multikey).unwrap();
    assert_eq!(Ok(Some(CString::new("ok").unwrap())), d2.get("b", None));
    assert_eq!(
        Ok(Some(vec![
            CString::new("ok").unwrap(),
            CString::new("test").unwrap(),
        ])),
        d2.get_all("b", None)
    );
    d.set_int("i", 17, None).unwrap();
    assert_eq!(Ok(Some(CString::new("17").unwrap())), d.get("i", None));
    d.set("c", "test", AVDictFlags::Append).unwrap();
    d.set("c", "test2", AVDictFlags::Append).unwrap();
    assert_eq!(
        Ok(Some(CString::new("testtest2").unwrap())),
        d.get("c", None)
    );
    assert_eq!(3, d.len());
    assert_eq!(3, d2.len());
    let mut m = HashMap::new();
    m.insert("s", "f");
    m.insert("f", "dd");
    let mut d3 = AVDict::new();
    d3.from_map(&m, None).unwrap();
    assert_eq!(Ok(Some(CString::new("dd").unwrap())), d3.get("f", None));
    let d3 = AVDict::try_from(&m).unwrap();
    assert_eq!(Ok(Some(CString::new("dd").unwrap())), d3.get("f", None));
    let mut d4 = AVDict::new();
    d4.parse_string("a=b b=c", "=", " ", None).unwrap();
    assert_eq!(2, d4.len());
    assert_eq!(
        Ok(CString::new("a=b b=c").unwrap()),
        d4.get_string('=', ' ')
    );
    let mut it = d4.iter();
    assert_eq!(
        Some((CString::new("a").unwrap(), CString::new("b").unwrap())),
        it.next()
    );
}
