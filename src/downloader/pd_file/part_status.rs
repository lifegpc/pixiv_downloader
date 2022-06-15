use crate::downloader::pd_file::enums::PdFilePartStatus as PdFilePartStatus2;
use crate::downloader::pd_file::error::PdFileError;
use crate::ext::atomic::AtomicQuick;
use crate::ext::rw_lock::GetRwLock;
use crate::gettext;
use int_enum::IntEnum;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B30;
use std::fmt::Debug;
use std::fmt::Display;
use std::io::Read;
use std::io::Write;
use std::sync::atomic::AtomicI64;
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
        Self {
            t: String::from(t.as_ref()),
            v: v,
        }
    }
}

impl<T: Display> Display for OutOfBoundsError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Failed to set type {} with value {}",
            self.t.as_str(),
            self.v
        ))
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
        f.write_fmt(format_args!(
            "{{ status: {:?}, downloaded_size: {:?} }}",
            self.status(),
            self.downloaded_size()
        ))
    }
}

#[derive(Debug)]
/// The status of the each part in pd file
pub struct PdFilePartStatus {
    /// The internal type
    status: RwLock<PdFilePartStatusInternal>,
    /// Retry count
    _retry_count: AtomicI64,
}

impl PdFilePartStatus {
    /// Create a new [PdFilePartStatus]
    /// # paincs
    /// Will panic if internal errors happened.
    pub fn new() -> Self {
        let mut status = PdFilePartStatusInternal::new();
        status.set_status(PdFilePartStatus2::Waited);
        status.set_downloaded_size(0);
        Self {
            status: RwLock::new(status),
            _retry_count: AtomicI64::new(0),
        }
    }

    #[inline]
    /// Returns the downloaded size of this part
    pub fn downloaded_size(&self) -> u32 {
        self.status.get_ref().downloaded_size()
    }

    #[inline]
    /// Returns the current retry count.
    pub fn retry_count(&self) -> i64 {
        self._retry_count.qload()
    }

    #[inline]
    /// Returns the status of this part
    pub fn status(&self) -> PdFilePartStatus2 {
        self.status.get_ref().status()
    }

    /// Set the new downloaded size
    pub fn set_downloaded_size(&self, new_size: u32) -> Result<(), PdFileError> {
        match self.status.get_mut().set_downloaded_size_checked(new_size) {
            Ok(_) => Ok(()),
            Err(_) => Err(PdFileError::from(OutOfBoundsError::new("u32", new_size))),
        }
    }

    #[inline]
    /// Set the new retry count.
    pub fn set_retry_count(&self, count: i64) {
        self._retry_count.qstore(count)
    }

    /// Set the status of this part
    pub fn set_status(&self, status: PdFilePartStatus2) -> Result<(), PdFileError> {
        match self.status.get_mut().set_status_checked(status) {
            Ok(_) => Ok(()),
            Err(_) => Err(PdFileError::from(OutOfBoundsError::new(
                "PdFilePartStatus",
                status,
            ))),
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
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(
        bytes: &T,
        offset: usize,
    ) -> Result<Self, PdFileError> {
        let value = bytes.as_ref();
        if (value.len() - offset) < 4 {
            Err(gettext("At least 4 bytes is needed."))?;
        }
        let st = (value[offset] & 0xC0) / 0x40;
        let mut status = PdFilePartStatusInternal::new();
        status.set_status(PdFilePartStatus2::from_int(st)?);
        // Downloaded size
        let mut ds: u32 = (value[offset] & 0x3f) as u32;
        ds += (value[offset + 1] as u32) * 0x40;
        ds += (value[offset + 2] as u32) * 0x40_00;
        ds += (value[offset + 3] as u32) * 0x40_00_00;
        status.set_downloaded_size(ds);
        Ok(Self {
            status: RwLock::new(status),
            _retry_count: AtomicI64::new(0),
        })
    }

    /// Get bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut tmp: u8 = self.status().int_value() * 0x40;
        let ds = self.downloaded_size();
        tmp += (ds & 0x3f) as u8;
        data.push(tmp);
        data.push(((ds & 0x3fc0) / 0x40) as u8);
        data.push(((ds & 0x3f_c0_00) / 0x40_00) as u8);
        data.push(((ds & 0x3f_c0_00_00) / 0x40_00_00) as u8);
        data
    }

    /// Create a new instance of the [PdFilePartStatus] from reader
    /// * `reader` - The reader which implement the [Read] trait
    ///
    /// Returns Error or [PdFilePartStatus] instance.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, PdFileError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Self::from_bytes(&buf, 0)
    }

    /// Write version bytes to writer.
    /// * `writer` - The writer which implement the [Write] trait
    ///
    /// Returns io Result.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_bytes())
    }
}

#[test]
fn test_part_status() {
    assert_eq!(std::mem::size_of::<PdFilePartStatusInternal>(), 4);
    let status = PdFilePartStatus::new();
    assert_eq!(status.status(), PdFilePartStatus2::Waited);
    assert_eq!(status.is_waited(), true);
    let status = PdFilePartStatus::from_bytes(&[132u8, 23, 3, 2], 0).unwrap();
    assert_eq!(status.is_downloaded(), true);
    assert_eq!(status.downloaded_size(), 0x80c5c4);
    assert_eq!(status.to_bytes(), vec![132, 23, 3, 2]);
    status.set_downloaded_size(0x323133).unwrap();
    assert_eq!(status.downloaded_size(), 0x323133);
    assert_eq!(status.to_bytes(), vec![179, 196, 200, 0]);
    assert_eq!(status.retry_count(), 0);
    status.set_retry_count(3);
    assert_eq!(status.retry_count(), 3);
}
