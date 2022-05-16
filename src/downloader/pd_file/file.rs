use crate::downloader::pd_file::error::PdFileError;
use crate::downloader::pd_file::enums::PdFileStatus;
use crate::downloader::pd_file::enums::PdFileType;
use crate::downloader::pd_file::version::PdFileVersion;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::try_err::TryErr;
use crate::ext::try_err::TryErr2;
use crate::gettext;
use int_enum::IntEnum;
use std::convert::AsRef;
use std::fs::File;
#[cfg(test)]
use std::fs::metadata;
use std::fs::remove_file;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Drop;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

lazy_static! {
    #[doc(hidden)]
    static ref MAGIC_WORDS: Vec<u8> = vec![0x50, 0x44, 0xff, 0xff];
}

/// The pd file
pub struct PdFile {
    /// The version of the current file.
    version: PdFileVersion,
    /// Needed to save to file.
    need_saved: AtomicBool,
    /// The file handle of the pd file.
    file: RwLock<Option<File>>,
    /// The file path of the pd file.
    file_path: RwLock<Option<PathBuf>>,
    /// The file name
    file_name: RwLock<Option<String>>,
    /// The status of the downloaded file.
    status: RwLock<PdFileStatus>,
    /// The type of the downloader.
    ftype: RwLock<PdFileType>,
    /// The target size of the file. If unknown, set this to 0.
    file_size: AtomicU64,
    /// The size of the downloaded data.
    downloaded_file_size: AtomicU64,
    /// The size of the each part. Ignored in single thread mode.
    part_size: AtomicU32,
}

impl PdFile {
    /// Create a new instance of the [PdFile]
    pub fn new() -> Self {
        Self {
            version: PdFileVersion::new(1, 0),
            need_saved: AtomicBool::new(false),
            file: RwLock::new(None),
            file_path: RwLock::new(None),
            file_name: RwLock::new(None),
            status: RwLock::new(PdFileStatus::Started),
            ftype: RwLock::new(PdFileType::SingleThread),
            file_size: AtomicU64::new(0),
            downloaded_file_size: AtomicU64::new(0),
            part_size: AtomicU32::new(0),
        }
    }

    /// Close the file.
    /// This function will return error if write failed.
    /// If you want to force close the file. Please use [Self::force_close()].
    pub fn close(&self) -> Result<(), PdFileError> {
        if self.is_need_saved() {
            self.write()?;
        }
        self.force_close();
        Ok(())
    }

    /// Force close the file
    pub fn force_close(&self) {
        let mut f = self.file.get_mut();
        match f.as_mut() {
            Some(_) => { f.take(); }
            None => {}
        }
    }

    #[inline]
    /// Returns true if the download is completed.
    pub fn is_completed(&self) -> bool {
        self.status.get_ref().is_completed()
    }

    #[inline]
    /// Returns true if needed to save to file.
    fn is_need_saved(&self) -> bool {
        self.need_saved.load(Ordering::Relaxed)
    }

    /// Create a new file and prepare to write data to it.
    /// If file alreay exists, will remove it first.
    /// * `path` - The path to the pd file.
    pub fn open_with_create_file<P: AsRef<Path> + ?Sized>(&self, path: &P) -> Result<(), PdFileError> {
        let p = path.as_ref();
        if p.exists() {
            remove_file(p)?;
        }
        let f = File::create(p)?;
        self.file.get_mut().replace(f);
        self.file_path.get_mut().replace(PathBuf::from(p));
        self.need_saved.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Remove file in [Self::file_path]
    fn remove_pd_file(&self) -> Result<(), PdFileError> {
        match self.file_path.get_ref().as_ref() {
            Some(pb) => {
                if pb.exists() {
                    remove_file(pb)?;
                }
                Ok(())
            }
            None => { Ok(()) }
        }
    }

    /// Remove file in [Self::file_path] and if error occered print that error.
    fn remove_pd_file_with_err_msg(&self) {
        match self.remove_pd_file() {
            Ok(_) => {}
            Err(e) => {
                println!("{} {}", gettext("Failed to remove file: "), e);
            }
        }
    }

    /// Set the file name
    /// * `file_name` - The file name. Should not be empty.
    pub fn set_file_name<S: AsRef<str> + ?Sized>(&self, file_name: &S) -> Result<(), PdFileError> {
        let fname = file_name.as_ref();
        if fname.is_empty() {
            Err(gettext("File name should not be empty."))?
        } else {
            self.file_name.get_mut().replace(String::from(fname));
            self.need_saved.store(true, Ordering::Relaxed);
            // Rewrite all datas.
            self.write()?;
            Ok(())
        }
    }

    /// Write all data to the file.
    pub fn write(&self) -> Result<(), PdFileError> {
        let mut f = self.file.get_mut();
        let mut f = f.as_mut().try_err(gettext("The file is not opened."))?;
        f.seek(SeekFrom::Start(0))?;
        f.write_all(&MAGIC_WORDS)?;
        self.version.write_to(&mut f)?;
        let file_name = self.file_name.get_ref().try_err2(gettext("File name is not set."))?;
        let file_name = file_name.as_bytes();
        f.write_all(&(file_name.len() as u32).to_le_bytes())?;
        f.write_all(&self.status.get_ref().int_value().to_le_bytes())?;
        let ftype = self.ftype.get_ref();
        f.write_all(&ftype.int_value().to_le_bytes())?;
        f.write_all(&self.file_size.load(Ordering::Relaxed).to_le_bytes())?;
        f.write_all(&self.downloaded_file_size.load(Ordering::Relaxed).to_le_bytes())?;
        let part_size = if ftype.is_multi() {
            self.part_size.load(Ordering::Relaxed)
        } else {
            0
        };
        f.write_all(&part_size.to_le_bytes())?;
        f.write_all(file_name)?;
        self.need_saved.store(false, Ordering::Relaxed);
        Ok(())
    }
}

impl Drop for PdFile {
    fn drop(&mut self) {
        if self.is_completed() {
            self.force_close();
            self.remove_pd_file_with_err_msg();
            return;
        }
        if self.is_need_saved() {
            match self.write() {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    self.force_close();
                    self.remove_pd_file_with_err_msg();
                }
            }
        };
        self.force_close();
    }
}

#[cfg(test)]
fn check_file_size<P: AsRef<Path> + ?Sized>(path: &P, size: u64) -> Result<(), PdFileError> {
    let m = metadata(path)?;
    assert!(m.len() == size);
    Ok(())
}

#[test]
fn test_pd_file() -> Result<(), PdFileError> {
    {
        let f = PdFile::new();
        f.open_with_create_file("test/a.pd")?;
        f.set_file_name("a")?;
    }
    check_file_size("test/a.pd", 33)?;
    Ok(())
}
