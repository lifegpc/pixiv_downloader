use crate::downloader::pd_file::error::PdFileError;
use crate::downloader::pd_file::enums::PdFileResult;
use crate::downloader::pd_file::enums::PdFileStatus;
use crate::downloader::pd_file::enums::PdFileType;
use crate::downloader::pd_file::part_status::PdFilePartStatus;
use crate::downloader::pd_file::version::PdFileVersion;
use crate::ext::atomic::AtomicQuick;
use crate::ext::io::StructRead;
use crate::ext::io::StructWrite;
use crate::ext::replace::ReplaceWith2;
use crate::ext::rw_lock::GetRwLock;
use crate::ext::try_err::TryErr;
use crate::ext::try_err::TryErr2;
use crate::gettext;
use int_enum::IntEnum;
use std::convert::AsRef;
use std::fs::File;
#[cfg(test)]
use std::fs::create_dir;
#[cfg(test)]
use std::fs::metadata;
use std::fs::remove_file;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Drop;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;

lazy_static! {
    #[doc(hidden)]
    static ref MAGIC_WORDS: Vec<u8> = vec![0x50, 0x44, 0xff, 0xff];
}

/// The offset of the status in pd file
const STATUS_OFFSET: SeekFrom = SeekFrom::Start(10);
/// The offset of the file_size in pd file 
const FILE_SIZE_OFFSET: SeekFrom = SeekFrom::Start(12);
/// The offset of the downloaded_file_size in pd file
const DOWNLOADED_FILE_SIZE_OFFSET: SeekFrom = SeekFrom::Start(20);

#[derive(Debug)]
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
    /// Only stored in memory.
    mem_only: AtomicBool,
    /// The status of the each part.
    part_datas: RwLock<Vec<Arc<PdFilePartStatus>>>,
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
            mem_only: AtomicBool::new(true),
            part_datas: RwLock::new(Vec::new()),
        }
    }

    /// Set the status to the initailzed status.
    pub fn clear(&self) -> Result<(), PdFileError> {
        self.status.replace_with2(PdFileStatus::Started);
        self.ftype.replace_with2(PdFileType::SingleThread);
        self.file_size.qstore(0);
        self.downloaded_file_size.qstore(0);
        self.part_size.qstore(0);
        self.part_datas.get_mut().clear();
        if !self.is_mem_only() {
            self.need_saved.qstore(true);
            // Remove old file and reopen
            self.reopen()?;
            // Rewrite all datas.
            self.write()?;
        }
        Ok(())
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

    /// Complete the download.
    /// After calling this function. The pd file will be removed.
    pub fn complete(&self) -> Result<(), PdFileError> {
        self.set_completed();
        if !self.is_mem_only() {
            self.force_close();
            self.remove_pd_file()?;
        }
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
    /// Returns the size of the downloaded data
    pub fn get_downloaded_file_size(&self) -> u64 {
        self.downloaded_file_size.qload()
    }

    #[inline]
    /// The target size of the file. 0 if unknown.
    pub fn get_file_size(&self) -> u64 {
        self.file_size.qload()
    }

    /// Return status data of a part
    /// * `index` - The part index
    pub fn get_part_data(&self, index: usize) -> Option<Arc<PdFilePartStatus>> {
        let datas = self.part_datas.get_ref();
        if index < datas.len() {
            Some(Arc::clone(&datas[index]))
        } else {
            None
        }
    }

    /// Increase the downloaded file size.
    /// * `size` - The file size want to added.
    /// 
    /// Returns the downloaded file size
    pub fn inc(&self, size: u64) -> Result<u64, PdFileError> {
        let mut downloaded_size = self.downloaded_file_size.qload();
        downloaded_size += size;
        self.downloaded_file_size.qstore(downloaded_size);
        if !self.is_mem_only() {
            self.need_saved.qstore(true);
            let mut f = self.file.get_mut();
            let f = f.as_mut().unwrap();
            f.seek(DOWNLOADED_FILE_SIZE_OFFSET)?;
            f.write_le_u64(downloaded_size)?;
            self.need_saved.qstore(false);
        }
        Ok(downloaded_size)
    }

    #[inline]
    /// Returns true if the download is completed.
    pub fn is_completed(&self) -> bool {
        self.status.get_ref().is_completed()
    }

    #[inline]
    /// Returns true if the download is in progress.
    pub fn is_downloading(&self) -> bool {
        self.status.get_ref().is_downloading()
    }

    #[inline]
    /// Returns true if stored in memory only.
    fn is_mem_only(&self) -> bool {
        self.mem_only.qload()
    }

    #[inline]
    /// Returns true if is multiple thread mode.
    pub fn is_multi_threads(&self) -> bool {
        self.ftype.get_ref().is_multi()
    }

    #[inline]
    /// Returns true if needed to save to file.
    fn is_need_saved(&self) -> bool {
        self.need_saved.qload()
    }

    /// Open a new [PdFile] if download is needed.
    /// * `path` - The path of the file which want to download.
    pub fn open<P: AsRef<Path> + ?Sized>(path: &P) -> Result<PdFileResult, PdFileError> {
        let p = path.as_ref();
        let mut pb = PathBuf::from(p);
        let mut file_name = pb.file_name().try_err(gettext("Path need have a file name."))?.to_owned();
        file_name.push(".pd");
        pb.set_file_name(&file_name);
        if p.exists() {
            if pb.exists() {
                let f = Self::read_from_file(p)?;
                if f.is_completed() {
                    return Ok(PdFileResult::TargetExisted);
                }
                Ok(PdFileResult::ExistedOk(f))
            } else {
                Ok(PdFileResult::TargetExisted)
            }
        } else {
            let f = PdFile::new();
            f.open_with_create_file(&pb)?;
            f.set_file_name(p.file_name().try_err(gettext("Path need have a file name."))?.to_str().unwrap_or("(null)"))?;
            Ok(PdFileResult::Ok(f))
        }
    }

    /// Create a new [PdFile] instance from the pd file.
    /// * `path` - The path to the pd file.
    /// 
    /// Returns errors or a new instance.
    pub fn read_from_file<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self, PdFileError> {
        let p = path.as_ref();
        let mut f = File::open(p)?;
        f.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8, 0, 0, 0];
        f.read_exact(&mut buf)?;
        if MAGIC_WORDS.as_ref() == buf {
            return Err(PdFileError::InvalidPdFile);
        }
        let version = PdFileVersion::read_from(&mut f)?;
        if !version.is_supported() {
            return Err(PdFileError::Unsupported);
        }
        let file_name_len = f.read_le_u32()?;
        let status: PdFileStatus = PdFileStatus::from_int(f.read_le_u8()?)?;
        let ftype: PdFileType = PdFileType::from_int(f.read_le_u8()?)?;
        let file_size = f.read_le_u64()?;
        let downloaded_file_size = f.read_le_u64()?;
        let part_size = f.read_le_u32()?;
        let file_name = String::from_utf8(f.read_bytes(file_name_len as usize)?)?;
        let mut part_datas = Vec::new();
        if ftype.is_multi() && file_size != 0 && part_size != 0 {
            let part_counts = (file_size + (part_size as u64) - 1) / (part_size as u64);
            for _ in 0..part_counts {
                let data = PdFilePartStatus::read_from(&mut f)?;
                part_datas.push(Arc::new(data));
            }
        }
        Ok(Self {
            version,
            need_saved: AtomicBool::new(false),
            file: RwLock::new(Some(f)),
            file_path: RwLock::new(Some(p.to_path_buf())),
            file_name: RwLock::new(Some(file_name)),
            status: RwLock::new(status),
            ftype: RwLock::new(ftype),
            file_size: AtomicU64::new(file_size),
            downloaded_file_size: AtomicU64::new(downloaded_file_size),
            part_size: AtomicU32::new(part_size),
            mem_only: AtomicBool::new(false),
            part_datas: RwLock::new(part_datas),
        })
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
        self.mem_only.qstore(false);
        self.need_saved.qstore(true);
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

    /// Reopen file with remove old file
    pub fn reopen(&self) -> Result<(), PdFileError> {
        let mut f = self.file.get_mut();
        if f.is_none() {
            return Ok(());
        }
        remove_file(self.file_path.get_ref().as_ref().unwrap())?;
        f.take();
        f.replace(File::create(self.file_path.get_ref().as_ref().unwrap())?);
        Ok(())
    }

    #[inline]
    /// Set status to alreay downloaded.
    fn set_completed(&self) {
        self.status.replace_with2(PdFileStatus::Downloaded);
    }

    #[inline]
    /// Set status to downloading.
    fn set_downloading(&self) {
        self.status.replace_with2(PdFileStatus::Downloading);
    }

    /// Set the file name
    /// * `file_name` - The file name. Should not be empty.
    pub fn set_file_name<S: AsRef<str> + ?Sized>(&self, file_name: &S) -> Result<(), PdFileError> {
        let fname = file_name.as_ref();
        if fname.is_empty() {
            Err(gettext("File name should not be empty."))?
        } else {
            self.file_name.get_mut().replace(String::from(fname));
            if !self.is_mem_only() {
                self.need_saved.qstore(true);
                self.reopen()?;
                // Rewrite all datas.
                self.write()?;
            }
            Ok(())
        }
    }

    /// Set the target size of the file. If unknown, set this to 0.
    /// * `file_size` - The target size of the file.
    /// 
    /// This will also set the status to downloading.
    pub fn set_file_size(&self, file_size: u64) -> Result<(), PdFileError> {
        self.file_size.qstore(file_size);
        if !self.is_mem_only() {
            self.need_saved.qstore(true);
            let mut f = self.file.get_mut();
            let f = f.as_mut().unwrap();
            f.seek(FILE_SIZE_OFFSET)?;
            f.write_le_u64(file_size)?;
            self.need_saved.qstore(false);
        }
        self.set_downloading();
        if !self.is_mem_only() {
            self.need_saved.qstore(true);
            let mut f = self.file.get_mut();
            let f = f.as_mut().unwrap();
            f.seek(STATUS_OFFSET)?;
            f.write_le_u8(self.status.get_ref().int_value())?;
            self.need_saved.qstore(false);
        }
        Ok(())
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
        f.write_le_u32(file_name.len() as u32)?;
        f.write_le_u8(self.status.get_ref().int_value())?;
        let ftype = self.ftype.get_ref();
        f.write_le_u8(ftype.int_value())?;
        let file_size = self.file_size.qload();
        f.write_le_u64(file_size)?;
        f.write_le_u64(self.downloaded_file_size.qload())?;
        let part_size = if ftype.is_multi() {
            self.part_size.qload()
        } else {
            0
        };
        f.write_le_u32(part_size)?;
        f.write_all(file_name)?;
        if ftype.is_multi() && file_size != 0 && part_size != 0 {
            let part_counts = (file_size + (part_size as u64) - 1) / (part_size as u64);
            let part_datas = self.part_datas.get_ref();
            if (part_counts as usize) <= part_datas.len() {
                for i in 0..part_counts {
                    part_datas[i as usize].write_to(&mut f)?;
                }
            }
        }
        self.need_saved.qstore(false);
        Ok(())
    }
}

impl Drop for PdFile {
    fn drop(&mut self) {
        if self.is_mem_only() {
            return;
        }
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
    let p = Path::new("./test");
    if !p.exists() {
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
    }
    {
        let f = PdFile::new();
        f.open_with_create_file("test/a.pd")?;
        f.set_file_name("a")?;
    }
    check_file_size("test/a.pd", 33)?;
    Ok(())
}
