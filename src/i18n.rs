#[cfg(windows)]
extern crate winapi;

use crate::utils::get_exe_path_else_current;
use gettext::Catalog;
use std::fs::File;

pub fn get_lang() -> String {
    let lan = std::env::var("LANG");
    match lan {
        Ok(l) => {
            if l.len() > 0 {
                return l;
            }
        }
        Err(_) => {}
    }
    #[cfg(windows)]
    {
        use std::alloc::alloc;
        use std::alloc::dealloc;
        use std::alloc::Layout;
        use std::mem::align_of;
        use std::mem::size_of;
        use std::ptr::null_mut;
        use winapi::um::stringapiset::WideCharToMultiByte;
        use winapi::um::winnls::GetUserDefaultLCID;
        use winapi::um::winnls::LCIDToLocaleName;
        use winapi::um::winnls::CP_UTF8;
        use winapi::um::winnls::WC_ERR_INVALID_CHARS;
        use winapi::um::winnt::LPSTR;
        use winapi::um::winnt::LPWSTR;
        use winapi::um::winnt::WCHAR;
        unsafe {
            let lcid = GetUserDefaultLCID();
            let len = LCIDToLocaleName(lcid, null_mut(), 0, 0);
            if len > 0 {
                let align = align_of::<WCHAR>();
                let s = size_of::<WCHAR>();
                let layout = Layout::from_size_align(len as usize * s, align);
                match layout {
                    Ok(lay) => {
                        let pstr = alloc(lay) as LPWSTR;
                        let re = LCIDToLocaleName(lcid, pstr, len, 0);
                        let mut result = String::from("");
                        if re > 0 {
                            let mlen = WideCharToMultiByte(
                                CP_UTF8,
                                WC_ERR_INVALID_CHARS,
                                pstr,
                                len,
                                null_mut(),
                                0,
                                null_mut(),
                                null_mut(),
                            );
                            if mlen > 0 {
                                let ali = align_of::<u8>();
                                let layout = Layout::from_size_align(mlen as usize, ali);
                                match layout {
                                    Ok(lay) => {
                                        let pmstr = alloc(lay) as LPSTR;
                                        let re = WideCharToMultiByte(
                                            CP_UTF8,
                                            WC_ERR_INVALID_CHARS,
                                            pstr,
                                            len,
                                            pmstr,
                                            mlen,
                                            null_mut(),
                                            null_mut(),
                                        );
                                        if re > 0 {
                                            result = String::from_raw_parts(
                                                pmstr as *mut u8,
                                                mlen as usize,
                                                mlen as usize,
                                            );
                                        } else {
                                            dealloc(pmstr as *mut u8, lay);
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                        dealloc(pstr as *mut u8, lay);
                        if result.len() > 0 {
                            return result;
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }
    return String::from("en-US");
}

pub struct I18n {
    catalog: Option<Catalog>,
}

fn open_mo_file(molang: &str) -> Option<File> {
    let pb = get_exe_path_else_current();
    let base = String::from("pixiv_downloader");
    let fname = base + "." + molang.replace("-", "_").as_str() + ".mo";
    let p = pb.join(fname);
    if p.exists() {
        let f = File::open(p);
        match f {
            Ok(f) => {
                return Some(f);
            }
            Err(_) => {}
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut p = pb.clone();
        p.pop();
        p.push(format!(
            "share/locale/{}/LC_MESSAGES/pixiv_downloader.mo",
            molang.replace("-", "_").as_str()
        ));
        if p.exists() {
            let f = File::open(p);
            match f {
                Ok(f) => {
                    return Some(f);
                }
                Err(_) => {}
            }
        }
    }
    return None;
}

impl I18n {
    pub fn new() -> I18n {
        let s = get_lang();
        let mut molang = s.as_str();
        if s.starts_with("zh") {
            let t = s.to_lowercase();
            if t == "zh-tw" || t == "zh-hant" || t == "zh-hk" {
                molang = "zh_TW";
            } else {
                molang = "zh_CN";
            }
        }
        let mut catalog: Option<Catalog> = None;
        let re = open_mo_file(molang);
        match re {
            Some(f) => {
                let re = Catalog::parse(f);
                match re {
                    Ok(c) => {
                        catalog = Some(c);
                    }
                    Err(_) => {}
                }
            }
            None => {}
        }
        return I18n { catalog: catalog };
    }
}

lazy_static! {
    #[doc(hidden)]
    static ref I18NT: I18n = I18n::new();
}

/// Get translation of text
/// * `s` - Origin text
pub fn gettext(s: &str) -> &str {
    match &I18NT.catalog {
        Some(c) => {
            return c.gettext(s);
        }
        None => {
            return s;
        }
    }
}
