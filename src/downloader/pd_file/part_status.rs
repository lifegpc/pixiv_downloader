use crate::downloader::pd_file::enums::PdFilePartStatus as PdFilePartStatus2;
use crate::downloader::pd_file::error::PdFileError;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use int_enum::IntEnum;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B30;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::RwLock;

/// The data is out of bounds.
#[derive(Clone, Debug)]
pub struct OutOfBoundsError<T> {
    /// Type name
    t: String,
    /// the value
    v: T,
}

impl<T> OutOfBoundsError<T> {
    pub fn new<S: AsRef<str> + ?Sized>(t: &S, v: T) -> Self {
        Self { t: String::from(t.as_ref()), v: v }
    }
}

impl<T: Display> Display for OutOfBoundsError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Failed to set type {} with value {}", self.t.as_str(), self.v))
    }
}

#[bitfield(bits = 32)]
#[derive(Clone)]
/// The status of the each part in pd file (For internal usage)
struct PdFilePartStatusInternal {
    #[bits = 2]
    status: PdFilePartStatus2,
    downloaded_size: B30,
}

impl Debug for PdFilePartStatusInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{{ status: {:?}, downloaded_size: {:?} }}", self.status(), self.downloaded_size()))
    }
}

#[derive(Debug)]
/// The status of the each part in pd file
pub struct PdFilePartStatus {
    status: RwLock<PdFilePartStatusInternal>,
}

impl PdFilePartStatus {
    /// Create a new [PdFilePartStatus]
    /// # paincs
    /// Will panic if internal errors happened.
    pub fn new() -> Self {
        let mut status = PdFilePartStatusInternal::new();
        status.set_status(PdFilePartStatus2::Waited);
        status.set_downloaded_size(0);
        Self { status: RwLock::new(status) }
    }

    /// Returns the status of this part
    pub fn status(&self) -> PdFilePartStatus2 {
        self.status.get_ref().status()
    }

    /// Set the status of this part
    pub fn set_status(&self, status: PdFilePartStatus2) -> Result<(), PdFileError> {
        match self.status.get_mut().set_status_checked(status) {
            Ok(_) => { Ok(()) }
            Err(_) => {
                Err(PdFileError::from(OutOfBoundsError::new("PdFilePartStatus", status)))
            }
        }
    }

    #[inline]
    /// Returns true if the download is waited
    pub fn is_waited(&self) -> bool {
        self.status().is_waited()
    }

    #[inline]
    /// Returns true if the download is started
    pub fn is_downloading(&self) -> bool {
        self.status().is_downloading()
    }

    #[inline]
    /// Returns true if the download is completed.
    pub fn is_downloaded(&self) -> bool {
        self.status().is_downloaded()
    }

    /// Create a new instance of the [PdFilePartStatus] from bytes.
    /// * `bytes` - The data
    /// * `offset` - The offset of the needed data
    /// 
    /// Returns a new instance if succeed otherwise a Error because the data is less than 4 bytes.
    /// # Panics
    /// Will panic if unwanted error occured
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(bytes: &T, offset: usize) -> Result<Self, PdFileError> {
        let value = bytes.as_ref();
        if (value.len() - offset) < 4 {
            Err(gettext("At least 4 bytes is needed."))?;
        }
        let st = (value[offset] & 0xC0) / 0x20;
        let mut status = PdFilePartStatusInternal::new();
        status.set_status(PdFilePartStatus2::from_int(st)?);
        // #TODO
        Ok(Self { status: RwLock::new(status) })
    }
}

#[test]
fn test_part_status() {
    assert_eq!(std::mem::size_of::<PdFilePartStatusInternal>(), 4);
    let status = PdFilePartStatus::new();
    assert_eq!(status.status(), PdFilePartStatus2::Waited);
    assert_eq!(status.is_waited(), true);
    let status = PdFilePartStatus::from_bytes(&[80u8, 0, 0, 0], 0).unwrap();
    assert_eq!(status.is_downloaded(), true);
}
