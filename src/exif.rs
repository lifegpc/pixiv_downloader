use crate::_exif;
use c_fixed_string::CFixedStr;
use int_enum::IntEnum;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(test)]
use std::fs::copy;
#[cfg(test)]
use std::fs::create_dir;
use std::ops::Drop;
use std::os::raw::c_long;
use std::path::Path;

/// Used primarily as identifiers when creating ExifValue
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum ExifTypeID {
    /// Exif BYTE type, 8-bit unsigned integer.
    BYTE = 1,
    /// Exif ASCII type, 8-bit byte.
    AsciiString = 2,
    /// Exif SHORT type, 16-bit (2-byte) unsigned integer.
    UShort = 3,
    /// Exif LONG type, 32-bit (4-byte) unsigned integer.
    ULong = 4,
    /// Exif RATIONAL type, two LONGs: numerator and denumerator of a fraction.
    URational = 5,
    /// Exif SBYTE type, an 8-bit signed (twos-complement) integer.
    SBYTE = 6,
    /// Exif UNDEFINED type, an 8-bit byte that may contain anything.
    Undefined = 7,
    /// Exif SSHORT type, a 16-bit (2-byte) signed (twos-complement) integer.
    SShort = 8,
    /// Exif SLONG type, a 32-bit (4-byte) signed (twos-complement) integer.
    SLong = 9,
    /// Exif SRATIONAL type, two SLONGs: numerator and denominator of a fraction.
    SRational = 10,
    /// TIFF FLOAT type, single precision (4-byte) IEEE format.
    TiffFloat = 11,
    /// TIFF DOUBLE type, double precision (8-byte) IEEE format.
    TiffDouble = 12,
    /// TIFF IFD type, 32-bit (4-byte) unsigned integer.
    TiffIfd = 13,
    /// Exif LONG LONG type, 64-bit (8-byte) unsigned integer.
    ULongLong = 16,
    /// Exif LONG LONG type, 64-bit (8-byte) signed integer.
    SLongLong = 17,
    /// TIFF IFD type, 64-bit (8-byte) unsigned integer.
    TiffIfd8 = 18,
    /// IPTC string type.
    String = 0x10000,
    /// IPTC date type.
    Date = 0x10001,
    /// IPTC time type.
    Time = 0x10002,
    /// Exiv2 type for the Exif user comment.
    Comment = 0x10003,
    /// Exiv2 type for a CIFF directory.
    Directory = 0x10004,
    /// XMP text type.
    XmpText = 0x10005,
    /// XMP alternative type.
    XmpAlt = 0x10006,
    /// XMP bag type.
    XmpBag = 0x10007,
    /// XMP sequence type.
    XmpSeq = 0x10008,
    /// XMP language alternative type.
    LangAlt = 0x10009,
    /// Invalid type id.
    InvalidTypeId = 0x1fffe,
    /// Last type id.
    LastTypeId = 0x1ffff,
}

/// Type to express the byte orde
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum ExifByteOrder {
    Invalid = 0,
    Little = 1,
    Big = 2,
}

/// Exif Key<br>
/// See [all available keys](https://exiv2.org/tags.html).
pub struct ExifKey {
    key: *mut _exif::ExifKey,
}

/// Return raw pointer of the handle
pub trait ToRawHandle<T> {
    /// Return raw pointer of the handle
    unsafe fn to_raw_handle(&self) -> *mut T;
}

impl TryFrom<CString> for ExifKey {
    type Error = ();
    fn try_from(value: CString) -> Result<Self, Self::Error> {
        let ptr = value.as_ptr();
        let re = unsafe { _exif::exif_create_key_by_key(ptr) };
        if re.is_null() {
            return Err(());
        }
        Ok(Self { key: re })
    }
}

impl TryFrom<&str> for ExifKey {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let p = CString::new(value);
        if p.is_err() {
            return Err(());
        }
        Self::try_from(p.unwrap())
    }
}

impl Drop for ExifKey {
    fn drop(&mut self) {
        if !self.key.is_null() {
            unsafe { _exif::exif_free_key(self.key) };
        }
    }
}

impl ToRawHandle<_exif::ExifKey> for ExifKey {
    unsafe fn to_raw_handle(&self) -> *mut _exif::ExifKey {
        self.key
    }
}

#[allow(dead_code)]
impl ExifKey {
    /// Create an Exif key from the tag number and group name
    pub fn from_id(id: u16, group_name: &str) -> Result<Self, ()> {
        let p = CString::new(group_name);
        if p.is_err() {
            return Err(());
        }
        let p = p.unwrap();
        let ptr = p.as_ptr();
        let re = unsafe { _exif::exif_create_key_by_id(id, ptr) };
        if re.is_null() {
            return Err(());
        }
        Ok(Self { key: re })
    }

    /// Return the key of the metadatum as a string.
    /// The key is of the form `familyName.groupName.tagName`
    /// # Note
    /// However that the key is not necessarily unique, e.g., an ExifData may contain multiple metadata with the same key.
    pub fn key(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_key(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return an identifier for the type of metadata (the first part of the key)
    pub fn family_name(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_family_name(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return the name of the group (the second part of the key)
    pub fn group_name(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_group_name(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return the name of the tag (which is also the third part of the key)
    pub fn tag_name(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_tag_name(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return the tag number.
    pub fn tag(&self) -> Option<u16> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_tag_tag(self.key) };
        if r == 65535 {
            None
        } else {
            Some(r)
        }
    }

    /// Return a label for the tag.
    pub fn tag_label(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_tag_label(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return the tag description.
    pub fn tag_desc(&self) -> Option<String> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_tag_desc(self.key) };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        let s = s.to_str();
        if s.is_err() {
            return None;
        }
        let s = s.unwrap();
        Some(s.to_owned())
    }

    /// Return the default type id for this tag.
    pub fn default_typeid(&self) -> Option<ExifTypeID> {
        if self.key.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_key_default_type_id(self.key) };
        if r == -1 {
            return None;
        }
        let re = ExifTypeID::from_int(r);
        if re.is_err() {
            return None;
        }
        Some(re.unwrap())
    }
}

/// Common interface for all types of values used with metadata.
pub struct ExifValue {
    value: *mut _exif::ExifValue,
}

impl TryFrom<ExifTypeID> for ExifValue {
    type Error = ();
    fn try_from(value: ExifTypeID) -> Result<Self, Self::Error> {
        let d = value.int_value();
        let r = unsafe { _exif::exif_create_value(d) };
        if r.is_null() {
            return Err(());
        }
        Ok(Self { value: r })
    }
}

impl TryFrom<i32> for ExifValue {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let e = ExifTypeID::from_int(value);
        if e.is_err() {
            return Err(());
        }
        Self::try_from(e.unwrap())
    }
}

impl Drop for ExifValue {
    fn drop(&mut self) {
        if !self.value.is_null() {
            unsafe { _exif::exif_free_value(self.value) };
        }
    }
}

impl ToRawHandle<_exif::ExifValue> for ExifValue {
    unsafe fn to_raw_handle(&self) -> *mut _exif::ExifValue {
        self.value
    }
}

#[allow(dead_code)]
impl ExifValue {
    /// Return the type identifier (Exif data format type).
    pub fn type_id(&self) -> Option<ExifTypeID> {
        if self.value.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_value_type_id(self.value) };
        if r == 0 {
            return None;
        }
        let re = ExifTypeID::from_int(r);
        if re.is_err() {
            return None;
        }
        Some(re.unwrap())
    }

    /// Return the number of components of the value.
    pub fn count(&self) -> Option<usize> {
        if self.value.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_value_count(self.value) };
        if r < 0 {
            return None;
        }
        Some(r as usize)
    }

    /// Return the size of the value in bytes.
    pub fn size(&self) -> Option<usize> {
        if self.value.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_value_size(self.value) };
        if r < 0 {
            return None;
        }
        Some(r as usize)
    }

    /// Return the size of the data area, 0 if there is none.
    pub fn size_data_area(&self) -> Option<usize> {
        if self.value.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_get_value_size_data_area(self.value) };
        if r < 0 {
            return None;
        }
        Some(r as usize)
    }

    /// Read the value from a character buffer.
    /// * `buf` - Buffer
    /// * `byte_order` - Applicable byte order (little or big endian). Default: invaild.
    pub fn read(&mut self, buf: &[u8], byte_order: Option<ExifByteOrder>) -> Result<(), ()> {
        if self.value.is_null() {
            return Err(());
        }
        let buf_len = buf.len() as c_long;
        let order = match byte_order {
            Some(o) => o,
            None => ExifByteOrder::Invalid,
        };
        let order = order.int_value();
        let ptr = buf.as_ptr();
        let r = unsafe { _exif::exif_value_read(self.value, ptr, buf_len, order) };
        if r == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Return the value / n-th component of the value as a string
    /// * `n` - specify the component, if None, return whole value
    pub fn to_string(&self, n: Option<usize>) -> Option<CString> {
        if self.value.is_null() {
            return None;
        }
        if n.is_some() {
            let c = self.count();
            if c.is_none() {
                return None;
            }
            if n.as_ref().unwrap() >= c.as_ref().unwrap() {
                return None;
            }
        }
        let mut size: Vec<_exif::size_t> = vec![0];
        let ptr = size.as_mut_ptr();
        if ptr.is_null() {
            return None;
        }
        let r = if n.is_none() {
            unsafe { _exif::exif_value_to_string(self.value, ptr) }
        } else {
            unsafe { _exif::exif_value_to_string2(self.value, ptr, n.unwrap() as c_long) }
        };
        if r.is_null() {
            return None;
        }
        let s = unsafe { CFixedStr::from_mut_ptr(r, size[0] as usize) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        if !self.ok() {
            return None;
        }
        Some(s.into_c_string())
    }

    /// Check the ok status indicator.
    /// After a to<Type> conversion, this indicator shows whether the conversion was successful.
    pub fn ok(&self) -> bool {
        if self.value.is_null() {
            return false;
        }
        let r = unsafe { _exif::exif_get_value_ok(self.value) };
        r != 0
    }

    /// Convert the n-th component of the value to a int64
    pub fn to_int64(&self, n: usize) -> Option<i64> {
        if self.value.is_null() {
            return None;
        }
        let c = self.count();
        if c.is_none() {
            return None;
        }
        let c = c.unwrap();
        if n >= c {
            return None;
        }
        let r = unsafe { _exif::exif_value_to_int64(self.value, n as c_long) };
        if !self.ok() {
            return None;
        }
        Some(r)
    }
}

/// A container for Exif data.
pub struct ExifData {
    data: *mut _exif::ExifData,
}

#[allow(dead_code)]
impl ExifData {
    /// Create a new container
    pub fn new() -> Result<Self, ()> {
        let d = unsafe { _exif::exif_data_new() };
        if d.is_null() {
            return Err(());
        }
        Ok(Self { data: d })
    }

    /// Add a data from the supplied key and value pair.
    /// No duplicate checks are performed, i.e., it is possible to add multiple metadata with the same key.
    pub fn add(&mut self, key: &ExifKey, value: &ExifValue) -> Result<(), ()> {
        let k = unsafe { key.to_raw_handle() };
        let v = unsafe { value.to_raw_handle() };
        if k.is_null() || v.is_null() {
            return Err(());
        }
        let r = unsafe { _exif::exif_data_add(self.data, k, v) };
        if r == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Delete all Exifdatum instances resulting in an empty container.
    /// Note that this also removes thumbnails.
    pub fn clear(&mut self) -> Result<(), ()> {
        if self.data.is_null() {
            return Err(());
        }
        let r = unsafe { _exif::exif_data_clear(self.data) };
        if r == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Get the number of metadata entries.
    pub fn count(&self) -> Option<usize> {
        if self.data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_get_count(self.data) };
        if r == -1 {
            return None;
        }
        Some(r as usize)
    }

    /// Return true if there is no Exif metadata.
    pub fn empty(&self) -> Option<bool> {
        if self.data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_is_empty(self.data) };
        if r == -1 {
            None
        } else if r == 0 {
            Some(false)
        } else {
            Some(true)
        }
    }
}

impl Drop for ExifData {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe { _exif::exif_free_data(self.data) };
        }
    }
}

impl ToRawHandle<_exif::ExifData> for ExifData {
    unsafe fn to_raw_handle(&self) -> *mut _exif::ExifData {
        self.data
    }
}

pub struct ExifImage {
    img: *mut _exif::ExifImage,
}

#[allow(dead_code)]
impl ExifImage {
    pub fn new<S: AsRef<OsStr> + ?Sized>(file_name: &S) -> Result<Self, ()> {
        let p = Path::new(file_name);
        if !p.exists() {
            return Err(());
        }
        let p = p.to_str();
        if p.is_none() {
            return Err(());
        }
        let p = CString::new(p.unwrap());
        if p.is_err() {
            return Err(());
        }
        let p = p.unwrap();
        let ptr = p.as_c_str().as_ptr();
        let f = unsafe { _exif::create_exif_image(ptr) };
        if f.is_null() {
            return Err(());
        }
        Ok(Self { img: f })
    }

    pub fn set_exif_data(&mut self, data: &ExifData) -> Result<(), ()> {
        if self.img.is_null() {
            return Err(());
        }
        let d = unsafe { data.to_raw_handle() };
        if d.is_null() {
            return Err(());
        }
        let d = unsafe { _exif::exif_image_set_exif_data(self.img, d) };
        if d == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_metadata(&mut self) -> Result<(), ()> {
        if self.img.is_null() {
            return Err(());
        }
        let d = unsafe { _exif::exif_image_write_metadata(self.img) };
        if d == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Drop for ExifImage {
    fn drop(&mut self) {
        if !self.img.is_null() {
            unsafe { _exif::free_exif_image(self.img) };
        }
    }
}

#[test]
fn test_exif_key() {
    let k = ExifKey::try_from("Exif.Image.XPTitle");
    assert!(k.is_ok());
    let k = k.unwrap();
    assert_eq!(Some(String::from("Exif.Image.XPTitle")), k.key());
    assert_eq!(Some(String::from("Exif")), k.family_name());
    assert_eq!(Some(String::from("Image")), k.group_name());
    assert_eq!(Some(String::from("XPTitle")), k.tag_name());
    assert_eq!(Some(40091), k.tag());
    assert_eq!(Some(String::from("Windows Title")), k.tag_label());
    assert_eq!(
        Some(String::from("Title tag used by Windows, encoded in UCS2")),
        k.tag_desc()
    );
    assert_eq!(Some(ExifTypeID::BYTE), k.default_typeid());
    let k2 = ExifKey::from_id(40091, "Image");
    assert!(k2.is_ok());
    let k2 = k2.unwrap();
    assert_eq!(Some(String::from("Exif.Image.XPTitle")), k2.key());
}

#[test]
fn test_exif_value() {
    let v = ExifValue::try_from(ExifTypeID::BYTE);
    assert!(v.is_ok());
    let mut v = v.unwrap();
    assert_eq!(Some(ExifTypeID::BYTE), v.type_id());
    assert_eq!(Some(0), v.count());
    assert_eq!(Some(0), v.size());
    assert_eq!(Some(0), v.size_data_area());
    assert!(v.read("test".as_bytes(), None).is_ok());
    assert_eq!(Some(4), v.count());
    assert_eq!(Some(4), v.size());
    let c = CString::new("116 101 115 116").unwrap();
    assert_eq!(Some(c), v.to_string(None));
    assert_eq!(Some(4), v.count());
    assert_eq!(Some(4), v.size());
    assert_eq!(Some(CString::new("116").unwrap()), v.to_string(Some(0)));
    let mut v2 = ExifValue::try_from(ExifTypeID::SLongLong).unwrap();
    v2.read(&(102345 as i64).to_le_bytes(), Some(ExifByteOrder::Little))
        .unwrap();
    assert_eq!(Some(8), v2.count());
}

#[test]
fn test_exif_data() {
    let mut d = ExifData::new().unwrap();
    assert_eq!(Some(0), d.count());
    assert_eq!(Some(true), d.empty());
    let k = ExifKey::try_from("Exif.Image.XPTitle").unwrap();
    let mut v = ExifValue::try_from(ExifTypeID::BYTE).unwrap();
    v.read("test".as_bytes(), None).unwrap();
    d.add(&k, &v).unwrap();
    assert_eq!(Some(1), d.count());
    assert_eq!(Some(false), d.empty());
    d.clear().unwrap();
    assert_eq!(Some(0), d.count());
    assert_eq!(Some(true), d.empty());
}

#[test]
fn test_exif_image() {
    let p = Path::new("./test");
    if !p.exists() {
        create_dir("./test").unwrap();
    }
    let target = "./test/夏のチマメ隊🏖️_91055644_p0.jpg";
    copy("./testdata/夏のチマメ隊🏖️_91055644_p0.jpg", target).unwrap();
    let mut d = ExifData::new().unwrap();
    let k = ExifKey::try_from("Exif.Image.ImageDescription").unwrap();
    let mut v = ExifValue::try_from(ExifTypeID::AsciiString).unwrap();
    v.read("夏のチマメ隊🏖️".as_bytes(), None).unwrap();
    d.add(&k, &v).unwrap();
    {
        let mut img = ExifImage::new(target).unwrap();
        img.set_exif_data(&d).unwrap();
        img.write_metadata().unwrap();
    }
}
