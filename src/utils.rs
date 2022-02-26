use std::env;
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
