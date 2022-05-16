use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::convert::AsRef;
use std::convert::TryFrom;
use std::io::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Version of the pd file
pub struct PdFileVersion {
    /// Major verson
    major: u8,
    /// Minor version
    minor: u8,
}

impl PdFileVersion {
    /// Create a new instance of the [PdFileVersion]
    /// * `major` - major version
    /// * `minor` - minor version
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    /// Create a new instance of the [PdFileVersion] from bytes.
    /// * `bytes` - The data
    /// * `offset` - The offset of the needed data
    /// 
    /// Returns a new instance if succeed otherwise a Error because the data is less than 2 bytes.
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(bytes: &T, offset: usize) -> Result<Self, ()> {
        let value = bytes.as_ref();
        if (value.len() - offset) < 2 {
            Err(())
        } else {
            Ok(Self::new(value[offset], value[offset + 1]))
        }
    }

    /// Get version bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.major);
        data.push(self.minor);
        data
    }

    /// Write version bytes to writer.
    /// * `writer` - The writer which implement the [Write] trait
    /// 
    /// Returns io Result.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_bytes())
    }
}

impl TryFrom<&[u8]> for PdFileVersion {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value, 0)
    }
}

impl Ord for PdFileVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let r = self.major.cmp(&other.major);
        if r.is_eq() {
            self.minor.cmp(&other.minor)
        } else {
            r
        }
    }
}

impl PartialOrd for PdFileVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let r = self.major.cmp(&other.major);
        if r.is_eq() {
            Some(self.minor.cmp(&other.minor))
        } else {
            Some(r)
        }
    }
}

/// # Note
/// If [Self::from_bytes()] returns error, assert it to false.
impl<T: AsRef<[u8]> + ?Sized> PartialEq<T> for PdFileVersion {
    fn eq(&self, other: &T) -> bool {
        match Self::from_bytes(other, 0) {
            Ok(v) => { v == *self }
            Err(_) => { false }
        }
    }
}

/// # Note
/// If [Self::from_bytes()] returns error, assert it to false.
impl<T: AsRef<[u8]> + ?Sized> PartialOrd<T> for PdFileVersion {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match Self::from_bytes(other, 0) {
            Ok(v) => { Some(self.cmp(&v)) }
            Err(_) => { None }
        }
    }
}

#[test]
fn test_pd_file_version() {
    assert!(PdFileVersion::new(1, 0) != PdFileVersion::new(0, 1));
    assert!(PdFileVersion::new(1, 10) != PdFileVersion::new(11, 0));
    assert!(PdFileVersion::new(1, 10) == PdFileVersion::new(1, 10));
    assert!(PdFileVersion::from_bytes(&vec![1, 10], 0).unwrap() == PdFileVersion::new(1, 10));
    assert_eq!(PdFileVersion::new(1, 10).to_bytes(), vec![1, 10]);
    assert!(PdFileVersion::new(2, 1) > PdFileVersion::new(1, 3));
    assert!(PdFileVersion::new(1, 3) > PdFileVersion::new(1, 2));
    assert!(PdFileVersion::new(3, 2) >= PdFileVersion::new(3, 2));
    assert!(PdFileVersion::new(1, 4) < PdFileVersion::new(1, 11));
    assert!(PdFileVersion::new(1, 3) < PdFileVersion::new(2, 1));
    assert!(PdFileVersion::new(1, 2) == [1u8, 2]);
    assert!(PdFileVersion::new(1, 3) == vec![1u8, 3, 2]);
    assert!(PdFileVersion::new(1, 2) == [1, 2, 3]);
    assert!(PdFileVersion::new(1, 3) == vec![1, 3, 2]);
    assert!(PdFileVersion::new(1, 2) != [1]);
    assert!(PdFileVersion::new(2, 3) > [1, 10]);
    assert!(PdFileVersion::new(3, 1) < [10, 1]);
    assert!(PdFileVersion::new(3, 3) >= [3, 3]);
    assert!(!(PdFileVersion::new(2, 3) > [1]))
}
