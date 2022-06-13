use crate::ext::io::ClearFile;
use std::fs::File;
use std::fs::remove_file;
use std::io::Seek;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

/// A wrapper for [File], add support for clear file content.
pub struct LocalFile {
    /// The file.
    file: Option<File>,
    /// The path of the file.
    path: PathBuf,
}

impl LocalFile {
    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist, and will truncate it if it does.
    pub fn create<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let p = path.as_ref().to_owned();
        let f = File::create(&p)?;
        Ok(Self {
            file: Some(f),
            path: p,
        })
    }

    /// Attempts to open a file in read-only mode.
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let p = path.as_ref().to_owned();
        let f = File::open(&p)?;
        Ok(Self {
            file: Some(f),
            path: p,
        })
    }
}

impl ClearFile for LocalFile {
    fn clear_file(&mut self) -> std::io::Result<()> {
        self.file.take();
        remove_file(&self.path)?;
        self.file.replace(File::create(&self.path)?);
        Ok(())
    }
}

impl Deref for LocalFile {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        &self.file.as_ref().unwrap()
    }
}

impl Seek for LocalFile {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.file.as_ref().unwrap().seek(pos)
    }
}

impl Write for LocalFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.as_ref().unwrap().write(buf)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.file.as_ref().unwrap().write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.as_ref().unwrap().flush()
    }
}
