use crate::gettext;
use reqwest::IntoUrl;
use std::env;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Get executable location, if not found, return current directory (./)
pub fn get_exe_path_else_current() -> PathBuf {
    let re = env::current_exe();
    match re {
        Ok(pa) => {
            let mut p = pa.clone();
            p.pop();
            p
        }
        Err(_) => {
            let p = Path::new("./");
            p.to_path_buf()
        }
    }
}

pub fn check_file_exists(path: &str) -> bool {
    let p = Path::new(path);
    p.exists()
}

pub fn ask_need_overwrite(path: &str) -> bool {
    let s = gettext("Do you want to delete file \"<file>\"?").replace("<file>", path);
    print!("{}(y/n)", s.as_str());
    std::io::stdout().flush().unwrap();
    let mut d = String::from("");
    loop {
        let re = std::io::stdin().read_line(&mut d);
        if re.is_err() {
            continue;
        }
        let d = d.trim().to_lowercase();
        if d == "y" {
            return true;
        } else {
            return false;
        }
    }
}

/// Get file name from url.
pub fn get_file_name_from_url<U: IntoUrl>(url: U) -> Option<String> {
    let u = url.into_url();
    if u.is_err() {
        println!("{} {}", gettext("Can not parse URL:"), u.unwrap_err());
        return None;
    }
    let u = u.unwrap();
    let path = Path::new(u.path());
    let re = path.file_name();
    if re.is_none() {
        println!("{} {}", gettext("Failed to get file name from path:"), u.path());
        return None;
    }
    let r = re.unwrap().to_str();
    if r.is_none() {
        return None;
    }
    Some(String::from(r.unwrap()))
}
