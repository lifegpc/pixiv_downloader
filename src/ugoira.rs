#[cfg(feature = "ugoira")]
use crate::_ugoira;
#[cfg(feature = "avdict")]
use crate::avdict::AVDict;
#[cfg(feature = "avdict")]
use crate::avdict::AVDictCodeError;
use crate::ext::cstr::ToCStr;
use crate::ext::cstr::ToCStrError;
use crate::ext::json::ToJson;
#[cfg(feature = "ugoira")]
use crate::ext::rawhandle::ToRawHandle;
use crate::ext::subprocess::PopenAsyncExt;
use crate::ext::try_err::TryErr;
use crate::gettext;
use std::collections::HashMap;
use std::convert::AsRef;
use std::default::Default;
use std::ffi::CStr;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Display;
#[cfg(test)]
use std::fs::{create_dir, File};
use std::io::Read;
use std::ops::Drop;
use std::os::raw::c_int;
use std::os::raw::c_void;
#[cfg(test)]
use std::path::Path;
use std::str::FromStr;
use std::str::Utf8Error;
use subprocess::ExitStatus;
use subprocess::Popen;
use subprocess::PopenConfig;
use subprocess::Redirection;

const UGOIRA_OK: c_int = 0;
const UGOIRA_NULL_POINTER: c_int = 1;
const UGOIRA_ZIP: c_int = 2;
const UGOIRA_INVALID_MAX_FPS: c_int = 3;
const UGOIRA_INVALID_FRAMES: c_int = 4;
const UGOIRA_INVALID_CRF: c_int = 5;
const UGOIRA_REMOVE_OUTPUT_FILE_FAILED: c_int = 6;
const UGOIRA_OOM: c_int = 7;
const UGOIRA_NO_VIDEO_STREAM: c_int = 8;
const UGOIRA_NO_AVAILABLE_DECODER: c_int = 9;
const UGOIRA_NO_AVAILABLE_ENCODER: c_int = 10;
const UGOIRA_OPEN_FILE: c_int = 11;
const UGOIRA_UNABLE_SCALE: c_int = 12;
const UGOIRA_JSON_ERROR: c_int = 13;

#[derive(Debug, derive_more::From)]
pub enum UgoiraError {
    String(String),
    Utf8(Utf8Error),
    ToCStr(ToCStrError),
    #[cfg(feature = "avdict")]
    FfmpegError(AVDictCodeError),
    CodeError(UgoiraCodeError),
    #[cfg(feature = "ugoira")]
    ZipError(UgoiraZipError),
    #[cfg(feature = "ugoira")]
    ZipError2(UgoiraZipError2),
    Popen(subprocess::PopenError),
}

impl Display for UgoiraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_str(s),
            Self::Utf8(s) => f.write_fmt(format_args!(
                "{} {}",
                gettext("Failed to decode string with UTF-8:"),
                s
            )),
            Self::ToCStr(s) => f.write_fmt(format_args!("{}", s)),
            #[cfg(feature = "avdict")]
            Self::FfmpegError(s) => f.write_fmt(format_args!("{}", s)),
            Self::CodeError(s) => f.write_fmt(format_args!("{}", s)),
            #[cfg(feature = "ugoira")]
            Self::ZipError(s) => f.write_fmt(format_args!("{}", s)),
            #[cfg(feature = "ugoira")]
            Self::ZipError2(s) => f.write_fmt(format_args!("{}", s)),
            Self::Popen(p) => f.write_fmt(format_args!("{}", p)),
        }
    }
}

impl From<&str> for UgoiraError {
    fn from(s: &str) -> Self {
        Self::String(String::from(s))
    }
}

impl From<c_int> for UgoiraError {
    fn from(v: c_int) -> Self {
        if v < 0 {
            #[cfg(feature = "avdict")]
            return Self::FfmpegError(AVDictCodeError::from(v));
            #[cfg(not(feature = "avdict"))]
            Self::String(format!("Error code from ffmpeg: {}", v))
        } else {
            Self::CodeError(UgoiraCodeError::from(v))
        }
    }
}

#[cfg(feature = "ugoira")]
impl From<_ugoira::UgoiraError> for UgoiraError {
    fn from(v: _ugoira::UgoiraError) -> Self {
        if v.code < 0 {
            Self::FfmpegError(AVDictCodeError::from(v.code))
        } else if v.code == UGOIRA_ZIP {
            if v.zip_err != 0 {
                Self::ZipError(UgoiraZipError::from(v.zip_err))
            } else {
                Self::ZipError2(UgoiraZipError2::from(v.zip_err2))
            }
        } else {
            Self::CodeError(UgoiraCodeError::from(v.code))
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct UgoiraCodeError {
    code: c_int,
}

impl UgoiraCodeError {
    fn to_str(&self) -> &'static str {
        match self.code {
            UGOIRA_OK => "OK",
            UGOIRA_NULL_POINTER => gettext("Arguments have null pointers."),
            UGOIRA_ZIP => "Libzip error",
            UGOIRA_INVALID_MAX_FPS => gettext("Invalid max fps."),
            UGOIRA_INVALID_FRAMES => gettext("Invalid frames."),
            UGOIRA_INVALID_CRF => gettext("Invalid crf."),
            UGOIRA_REMOVE_OUTPUT_FILE_FAILED => gettext("Can not remove output file."),
            UGOIRA_OOM => gettext("Out of memory."),
            UGOIRA_NO_VIDEO_STREAM => gettext("No video stream available in the file."),
            UGOIRA_NO_AVAILABLE_DECODER => gettext("No available decoder."),
            UGOIRA_NO_AVAILABLE_ENCODER => gettext("No available encoder."),
            UGOIRA_OPEN_FILE => gettext("Failed to open output file."),
            UGOIRA_UNABLE_SCALE => gettext("Unable to scale image."),
            UGOIRA_JSON_ERROR => gettext("Failed to parse JSON file."),
            _ => gettext("Unknown error."),
        }
    }
}

impl Debug for UgoiraCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}({})", self.to_str(), self.code).as_str())
    }
}

impl Display for UgoiraCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl From<c_int> for UgoiraCodeError {
    fn from(v: c_int) -> Self {
        Self { code: v }
    }
}

#[cfg(feature = "ugoira")]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct UgoiraZipError {
    code: c_int,
}

#[cfg(feature = "ugoira")]
impl UgoiraZipError {
    pub fn to_str(&self) -> Result<String, UgoiraError> {
        let s = unsafe { _ugoira::ugoira_get_zip_err_msg(self.code) };
        if s.is_null() {
            Err(gettext("Out of memory."))?;
        }
        let ss = unsafe { CStr::from_ptr(s) };
        let ss = ss.to_owned();
        unsafe { _ugoira::ugoira_mfree(s as *mut c_void) };
        let re = ss.to_str()?;
        Ok(String::from(re))
    }
}

#[cfg(feature = "ugoira")]
impl Display for UgoiraZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.to_str()
                .unwrap_or(format!(
                    "{} {}",
                    gettext("Failed to get error message:"),
                    self.code
                ))
                .as_str(),
        )
    }
}

#[cfg(feature = "ugoira")]
impl From<c_int> for UgoiraZipError {
    fn from(v: c_int) -> Self {
        Self { code: v }
    }
}

#[cfg(feature = "ugoira")]
pub struct UgoiraZipError2 {
    err: *mut _ugoira::zip_error_t,
}

#[cfg(feature = "ugoira")]
impl UgoiraZipError2 {
    pub fn to_str(&self) -> Result<String, UgoiraError> {
        if self.err.is_null() {
            return Ok(String::from(gettext("No error.")));
        }
        let s = unsafe { _ugoira::ugoira_get_zip_err2_msg(self.err) };
        if s.is_null() {
            Err(gettext("Out of memory."))?;
        }
        let ss = unsafe { CStr::from_ptr(s) };
        let ss = ss.to_owned();
        unsafe { _ugoira::ugoira_mfree(s as *mut c_void) };
        let re = ss.to_str()?;
        Ok(String::from(re))
    }
}

#[cfg(feature = "ugoira")]
impl Debug for UgoiraZipError2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.err.is_null() {
            f.write_str("UgoiraError2 { None }")
        } else {
            let err = unsafe { *self.err };
            f.write_fmt(format_args!(
                "UgoiraZipError2 {{ {}, {} }}",
                err.sys_err, err.zip_err
            ))
        }
    }
}

#[cfg(feature = "ugoira")]
impl Display for UgoiraZipError2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.to_str()
                .unwrap_or(format!(
                    "{} {}, {}",
                    gettext("Failed to get error message:"),
                    unsafe { (*self.err).sys_err },
                    unsafe { (*self.err).zip_err }
                ))
                .as_str(),
        )
    }
}

#[cfg(feature = "ugoira")]
impl Drop for UgoiraZipError2 {
    fn drop(&mut self) {
        if !self.err.is_null() {
            unsafe { _ugoira::free_ugoira_error(self.err) };
            self.err = 0 as *mut _ugoira::zip_error_t;
        }
    }
}

#[cfg(feature = "ugoira")]
impl From<*mut _ugoira::zip_error_t> for UgoiraZipError2 {
    fn from(err: *mut _ugoira::zip_error_t) -> Self {
        Self { err }
    }
}

#[cfg(feature = "ugoira")]
impl PartialEq for UgoiraZipError2 {
    fn eq(&self, other: &Self) -> bool {
        if self.err.is_null() && other.err.is_null() {
            true
        } else if self.err.is_null() || other.err.is_null() {
            false
        } else {
            let e = unsafe { *self.err };
            let e2 = unsafe { *other.err };
            if e.sys_err == e2.sys_err && e.zip_err == e2.zip_err {
                true
            } else {
                false
            }
        }
    }
}

#[cfg(feature = "ugoira")]
unsafe impl Send for UgoiraZipError2 {}
#[cfg(feature = "ugoira")]
unsafe impl Sync for UgoiraZipError2 {}

#[cfg(feature = "ugoira")]
impl ToRawHandle<_ugoira::zip_error_t> for UgoiraZipError2 {
    unsafe fn to_raw_handle(&self) -> *mut _ugoira::zip_error_t {
        self.err
    }
}

#[cfg(feature = "ugoira")]
pub struct UgoiraFrames {
    head: *mut _ugoira::UgoiraFrame,
    tail: *mut _ugoira::UgoiraFrame,
}

#[cfg(feature = "ugoira")]
#[allow(dead_code)]
impl UgoiraFrames {
    pub fn new() -> Self {
        Self {
            head: 0 as *mut _ugoira::UgoiraFrame,
            tail: 0 as *mut _ugoira::UgoiraFrame,
        }
    }

    pub fn append<T: ToCStr>(&mut self, file: T, delay: f32) -> Result<(), UgoiraError> {
        let f = file.to_cstr()?;
        if delay <= 0f32 {
            Err(gettext("<sth> should be greater than <num>.")
                .replace("<sth>", gettext("Delay"))
                .replace("<num>", "0"))?;
        }
        let re = unsafe { _ugoira::new_ugoira_frame(f.as_ptr(), delay, self.tail) };
        if re.is_null() {
            Err(gettext("Out of memory."))?;
        }
        if self.head.is_null() {
            self.head = re;
        }
        self.tail = re;
        Ok(())
    }

    pub fn from_json<T: ToJson>(value: T) -> Result<Self, UgoiraError> {
        let obj = value
            .to_json()
            .try_err(gettext("Failed to get JSON object."))?;
        if !obj.is_array() {
            Err(gettext("Unsupported JSON type."))?;
        }
        let mut r = Self::new();
        for o in obj.members() {
            if !o.is_object() {
                Err(gettext("Unsupported JSON type."))?;
            }
            let file = o["file"].as_str().try_err(gettext("File is needed."))?;
            let delay = o["delay"].as_f32().try_err(gettext("Delay is needed."))?;
            r.append(file, delay)?;
        }
        Ok(r)
    }

    pub fn len(&self) -> usize {
        if self.head.is_null() {
            return 0;
        }
        let mut c = 1;
        let mut now = self.head;
        while unsafe { !(*now).next.is_null() } {
            now = unsafe { (*now).next };
            c += 1;
        }
        return c;
    }
}

#[cfg(feature = "ugoira")]
impl AsRef<Self> for UgoiraFrames {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[cfg(feature = "ugoira")]
impl Default for UgoiraFrames {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ugoira")]
impl Drop for UgoiraFrames {
    fn drop(&mut self) {
        if !self.head.is_null() {
            unsafe { _ugoira::free_ugoira_frames(self.head) };
            self.head = 0 as *mut _ugoira::UgoiraFrame;
        }
        self.tail = 0 as *mut _ugoira::UgoiraFrame;
    }
}

#[cfg(feature = "ugoira")]
impl ToRawHandle<_ugoira::UgoiraFrame> for UgoiraFrames {
    unsafe fn to_raw_handle(&self) -> *mut _ugoira::UgoiraFrame {
        self.head
    }
}

#[cfg(feature = "ugoira")]
impl ToRawHandle<_ugoira::AVDictionary> for AVDict {
    unsafe fn to_raw_handle(&self) -> *mut _ugoira::AVDictionary {
        self.m as *mut _ugoira::AVDictionary
    }
}

#[derive(Clone, Copy, Debug)]
/// H.264 profile
pub enum X264Profile {
    /// Selected by x264.
    Auto,
    /// No interlaced, No lossless.
    Baseline,
    /// No lossless.
    Main,
    /// No lossless.
    High,
    /// No lossless. Support for bit depth 8-10.
    High10,
    /// No lossless. Support for bit depth 8-10. Support for 4:2:0/4:2:2 chroma subsampling.
    High422,
    /// No lossless. Support for bit depth 8-10. Support for 4:2:0/4:2:2/4:4:4 chroma subsampling.
    High444,
}

impl X264Profile {
    pub fn as_str(&self) -> &'static str {
        match self {
            X264Profile::Auto => "",
            X264Profile::Baseline => "baseline",
            X264Profile::Main => "main",
            X264Profile::High => "high",
            X264Profile::High10 => "high10",
            X264Profile::High422 => "high422",
            X264Profile::High444 => "high444",
        }
    }

    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(self, X264Profile::Auto)
    }
}

impl AsRef<str> for X264Profile {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Default for X264Profile {
    fn default() -> Self {
        X264Profile::Auto
    }
}

impl FromStr for X264Profile {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "auto" => Ok(X264Profile::Auto),
            "baseline" => Ok(X264Profile::Baseline),
            "main" => Ok(X264Profile::Main),
            "high" => Ok(X264Profile::High),
            "high10" => Ok(X264Profile::High10),
            "high422" => Ok(X264Profile::High422),
            "high444" => Ok(X264Profile::High444),
            _ => Err(gettext("Unknown H.264 profile.")),
        }
    }
}

impl TryFrom<&str> for X264Profile {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        X264Profile::from_str(s)
    }
}

#[cfg(feature = "ugoira")]
pub fn convert_ugoira_to_mp4<
    S: AsRef<OsStr> + ?Sized,
    D: AsRef<OsStr> + ?Sized,
    F: AsRef<UgoiraFrames> + ?Sized,
    O: AsRef<AVDict> + ?Sized,
    M: AsRef<AVDict> + ?Sized,
>(
    src: &S,
    dest: &D,
    frames: &F,
    max_fps: f32,
    opts: &O,
    metadata: &M,
) -> Result<(), UgoiraError> {
    let src = src.as_ref();
    let dest = dest.as_ref();
    let frames = frames.as_ref();
    let opts = opts.as_ref();
    let metadata = metadata.as_ref();
    let src = src
        .to_str()
        .try_err(gettext("Failed to convert path."))?
        .to_cstr()?;
    let dest = dest
        .to_str()
        .try_err(gettext("Failed to convert path."))?
        .to_cstr()?;
    let re = unsafe {
        _ugoira::convert_ugoira_to_mp4(
            src.as_ptr(),
            dest.as_ptr(),
            frames.to_const_handle(),
            max_fps,
            opts.to_const_handle(),
            metadata.to_const_handle(),
        )
    };
    if re.code != 0 {
        Err(re)?;
    }
    Ok(())
}

pub async fn convert_ugoira_to_mp4_subprocess<
    B: AsRef<OsStr> + ?Sized,
    S: AsRef<OsStr> + ?Sized,
    D: AsRef<OsStr> + ?Sized,
    J: AsRef<OsStr> + ?Sized,
>(
    base: &B,
    src: &S,
    dest: &D,
    json: &J,
    max_fps: f32,
    metadata: HashMap<String, String>,
    force_yuv420p: bool,
    crf: Option<f32>,
    profile: Option<X264Profile>,
) -> Result<(), UgoiraError> {
    let mut argv: Vec<OsString> = Vec::with_capacity(5);
    argv.push(base.as_ref().to_owned());
    argv.push(src.as_ref().to_owned());
    argv.push(dest.as_ref().to_owned());
    argv.push(json.as_ref().to_owned());
    argv.push(format!("-M{}", max_fps).into());
    for (k, v) in metadata {
        argv.push("-m".into());
        argv.push(format!("{}={}", k, v).into());
    }
    if force_yuv420p {
        argv.push("-f".into());
    }
    match crf {
        Some(crf) => {
            argv.push("--crf".into());
            argv.push(crf.to_string().into());
        }
        None => {}
    }
    match profile {
        Some(p) => {
            if !p.is_auto() {
                argv.push(format!("-P{}", p.as_str()).into());
            }
        }
        None => {}
    }
    log::debug!(target: "ugoira_cli", "Command line: {:?}", argv);
    let mut p = Popen::create(
        &argv,
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::Pipe,
            stderr: Redirection::Merge,
            ..Default::default()
        },
    )?;
    let e = p.async_wait().await;
    let is_ok = match &e {
        ExitStatus::Exited(e) => *e == 0,
        _ => false,
    };
    match &mut p.stdout {
        Some(f) => {
            let mut s = String::new();
            match f.read_to_string(&mut s) {
                Ok(_) => {
                    if is_ok {
                        log::debug!(target: "ugoira_cli", "Output:\n{}", s);
                    } else {
                        log::info!(target: "ugoira_cli", "Output:\n{}", s);
                    }
                }
                Err(_) => {}
            }
        }
        None => {}
    }
    if !is_ok {
        match e {
            ExitStatus::Exited(e) => {
                return Err(UgoiraError::from(UgoiraCodeError::from(e as c_int)));
            }
            _ => {}
        }
        return Err(UgoiraError::from(format!("Unknown exit status: {:?}", e)));
    }
    Ok(())
}

#[cfg(all(feature = "ugoira", test))]
async fn get_ugoira_zip_error2() -> UgoiraZipError2 {
    let ugo = unsafe { _ugoira::new_ugoira_error() };
    if ugo.is_null() {
        panic!("Out of memory.");
    }
    UgoiraZipError2 { err: ugo }
}

#[cfg(feature = "ugoira")]
#[tokio::test]
async fn test_ugoira_zip_error2() {
    let task = tokio::spawn(get_ugoira_zip_error2());
    let re = task.await.unwrap();
    assert!(re.to_str().is_ok())
}

#[cfg(feature = "ugoira")]
#[test]
fn test_ugoira_frames() {
    let mut f = UgoiraFrames::new();
    assert_eq!(0, f.len());
    f.append("test.png", 32f32).unwrap();
    assert_eq!(1, f.len());
    f.append("test2.png", 31f32).unwrap();
    assert_eq!(2, f.len());
    f.append("fgng", 23f32).unwrap();
    assert_eq!(3, f.len());
    let f2 = UgoiraFrames::from_json(json::array![{"file": "a.jpg", "delay": 2}]).unwrap();
    assert_eq!(1, f2.len());
}

#[cfg(feature = "ugoira")]
#[test]
fn test_ugoira_zip_error() {
    let e = UgoiraZipError::from(3);
    assert!(e.to_str().is_ok())
}

#[cfg(feature = "ugoira")]
#[test]
fn test_convert_ugoira_to_mp4() -> Result<(), UgoiraError> {
    let frames_path = Path::new("./testdata/74841737_frames.json");
    if !frames_path.exists() {
        Err("Can not find frames file.")?;
    }
    let mut f = File::open(frames_path).unwrap();
    let mut s = String::from("");
    f.read_to_string(&mut s).unwrap();
    let o = json::parse(s.as_str()).unwrap();
    let frames = UgoiraFrames::from_json(o)?;
    assert_eq!(90, frames.len());
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let target = "./test/74841737.mp4";
    let mut metadata = AVDict::new();
    metadata.set("title", "動く nachoneko :3", None).unwrap();
    metadata.set("artist", "甘城なつき", None).unwrap();
    let options = AVDict::new();
    convert_ugoira_to_mp4(
        "./testdata/74841737_ugoira600x600.zip",
        target,
        &frames,
        60f32,
        &options,
        &metadata,
    )
}

#[proc_macros::async_timeout_test(120s)]
#[tokio::test(flavor = "multi_thread")]
async fn test_convert_ugoira_to_mp4_subprocess() -> Result<(), UgoiraError> {
    #[cfg(feature = "ugoira")]
    let base = crate::utils::get_exe_path_else_current()
        .join("../ugoira")
        .to_str()
        .unwrap()
        .to_owned();
    #[cfg(not(feature = "ugoira"))]
    let base = match std::env::var("UGOIRA") {
        Ok(b) => b,
        Err(_) => {
            println!("No ugoira location specified, skip test.");
            return Ok(());
        }
    };
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    let mut m = HashMap::new();
    m.insert(String::from("title"), String::from("動く nachoneko :3"));
    convert_ugoira_to_mp4_subprocess(
        &base,
        "./testdata/74841737_ugoira600x600.zip",
        "./test/74841737_sub.mp4",
        "./testdata/74841737_frames.json",
        60.0,
        m,
        false,
        None,
        None,
    )
    .await
}
