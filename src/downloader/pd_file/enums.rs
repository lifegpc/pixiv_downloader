use crate::downloader::pd_file::file::PdFile;
use int_enum::IntEnum;

/// The status of the downloaded file.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum PdFileStatus {
    /// The download is already started but the target size is unknown.
    Started = 0,
    /// The download is started and the tagret size is known.
    Downloading = 1,
    /// The download is completed.
    Downloaded = 2,
}

impl PdFileStatus {
    #[inline]
    /// Returns true if the download is completed.
    pub fn is_completed(&self) -> bool {
        *self == PdFileStatus::Downloaded
    }

    #[inline]
    /// Returns true if the download is in progress.
    pub fn is_downloading(&self) -> bool {
        *self == PdFileStatus::Downloading
    }
}

/// The type of the downloader.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum PdFileType {
    /// Download in single thread mode.
    SingleThread = 0,
    /// Download in multiple thread mode.
    MultiThread = 1,
}

#[derive(Debug)]
/// The result when try opening pd file.
pub enum PdFileResult {
    /// The pd file is not existed, and the new pd file is created.
    /// In this case, need download whole file.
    Ok(PdFile),
    /// The pd file is not existed but the target file is existed.
    /// In most case, this means the download already completed.
    TargetExisted,
    /// The pd file is existed.
    /// In this case, can continue to download.
    ExistedOk(PdFile),
}

impl PdFileType {
    #[inline]
    /// Returns true if is multiple thread mode.
    pub fn is_multi(&self) -> bool {
        *self == PdFileType::MultiThread
    }
}

#[test]
fn test_enums() {
    assert_eq!(PdFileStatus::Downloading.int_value().to_le_bytes(), [1]);
    assert_eq!(PdFileType::MultiThread.int_value().to_le_bytes(), [1]);
}
