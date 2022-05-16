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
