use crate::_exif;
use crate::_exif::ExifDataRef;
use crate::ext::rawhandle::FromRawHandle;
use crate::ext::rawhandle::ToRawHandle;
use c_fixed_string::CFixedStr;
use int_enum::IntEnum;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::borrow::ToOwned;
use std::clone::Clone;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(test)]
use std::fs::copy;
#[cfg(test)]
use std::fs::create_dir;
use std::ops::Deref;
use std::ops::DerefMut;
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

impl Clone for ExifKey {
    fn clone(&self) -> Self {
        let key = unsafe { _exif::exif_create_key_by_another(self.key) };
        if key.is_null() {
            panic!("Out of memory.");
        }
        Self { key }
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

    pub unsafe fn from_raw_pointer(data: *mut _exif::ExifData) -> Self {
        Self { data }
    }
}

impl Borrow<ExifDataRef> for ExifData {
    fn borrow(&self) -> &ExifDataRef {
        self.deref()
    }
}

impl BorrowMut<ExifDataRef> for ExifData {
    fn borrow_mut(&mut self) -> &mut ExifDataRef {
        self.deref_mut()
    }
}

impl Deref for ExifData {
    type Target = ExifDataRef;
    fn deref(&self) -> &Self::Target {
        unsafe {
            ExifDataRef::from_const_handle(
                _exif::exif_data_get_ref(self.to_raw_handle()) as *const ExifDataRef
            )
        }
    }
}

impl DerefMut for ExifData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { ExifDataRef::from_raw_handle(_exif::exif_data_get_ref(self.to_raw_handle())) }
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

#[allow(dead_code)]
impl ExifDataRef {
    /// Add a data from the supplied key and value pair.
    /// No duplicate checks are performed, i.e., it is possible to add multiple metadata with the same key.
    pub fn add(&mut self, key: &ExifKey, value: &ExifValue) -> Result<(), ()> {
        let data = unsafe { self.to_raw_handle() };
        let k = unsafe { key.to_raw_handle() };
        let v = unsafe { value.to_raw_handle() };
        if k.is_null() || v.is_null() || data.is_null() {
            return Err(());
        }
        let r = unsafe { _exif::exif_data_ref_add(data, k, v) };
        if r == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Delete all Exifdatum instances resulting in an empty container.
    /// Note that this also removes thumbnails.
    pub fn clear(&mut self) -> Result<(), ()> {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return Err(());
        }
        let r = unsafe { _exif::exif_data_ref_clear(data) };
        if r == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Get the number of metadata entries.
    pub fn count(&self) -> Option<usize> {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_ref_get_count(data) };
        if r == -1 {
            return None;
        }
        Some(r as usize)
    }

    /// Return true if there is no Exif metadata.
    pub fn empty(&self) -> Option<bool> {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_ref_is_empty(data) };
        if r == -1 {
            None
        } else if r == 0 {
            Some(false)
        } else {
            Some(true)
        }
    }
}

impl ToOwned for ExifDataRef {
    type Owned = ExifData;
    fn to_owned(&self) -> Self::Owned {
        let o = unsafe { self.to_raw_handle() };
        if o.is_null() {
            return ExifData::new().unwrap();
        }
        let r = unsafe { _exif::exif_data_ref_clone(o) };
        if r.is_null() {
            panic!("Failed to convert ExifDataRef to ExifData.");
        }
        unsafe { ExifData::from_raw_pointer(r) }
    }
}

impl ToRawHandle<ExifDataRef> for ExifDataRef {
    unsafe fn to_raw_handle(&self) -> *mut ExifDataRef {
        self.to_const_handle() as *mut ExifDataRef
    }
    unsafe fn to_const_handle(&self) -> *const ExifDataRef {
        self
    }
}

/// An image
pub struct ExifImage {
    img: *mut _exif::ExifImage,
}

#[allow(dead_code)]
impl ExifImage {
    /// Create a new image instance
    /// * `file_name` - File name
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

    /// Returns a read only [ExifData] ([ExifDataRef]) instance containing currently buffered Exif data.
    /// The Exif data in the returned instance will be written to the image when [Self::write_metadata()] is called.
    pub fn exif_data(&self) -> Option<&ExifDataRef> {
        if self.img.is_null() {
            return None;
        }
        let d = unsafe { _exif::exif_image_get_exif_data(self.img) };
        if d.is_null() {
            return None;
        }
        unsafe { Some(ExifDataRef::from_const_handle(d as *const ExifDataRef)) }
    }

    /// Returns an ExifData instance containing currently buffered Exif data.
    /// The Exif data in the returned instance will be written to the image when [Self::write_metadata()] is called.
    pub fn exif_data_as_mut(&mut self) -> Option<&mut ExifDataRef> {
        if self.img.is_null() {
            return None;
        }
        let d = unsafe { _exif::exif_image_get_exif_data(self.img) };
        if d.is_null() {
            return None;
        }
        unsafe { Some(ExifDataRef::from_raw_handle(d)) }
    }

    /// Read all metadata supported by a specific image format from the image. Before this method is called, the image metadata will be cleared.
    ///
    /// This method returns success even if no metadata is found in the image. Callers must therefore check the size of individual metadata types before accessing the data.
    pub fn read_metadata(&mut self) -> Result<(), ()> {
        if self.img.is_null() {
            return Err(());
        }
        let d = unsafe { _exif::exif_image_read_metadata(self.img) };
        if d == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Assign new Exif data.
    /// The new Exif data is not written to the image until the [Self::write_metadata()] method is called.
    /// * `data` - An [ExifData] instance holding Exif data to be copied
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

    /// Write metadata back to the image.
    ///
    /// All existing metadata sections in the image are either created, replaced, or erased.
    /// If values for a given metadata type have been assigned, a section for that metadata type will either be created or replaced.
    /// If no values have been assigned to a given metadata type, any exists section for that metadata type will be removed from the image.
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
            self.img = 0 as *mut _exif::ExifImage;
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
    let k3 = k2.clone();
    assert_eq!(Some(String::from("Exif.Image.XPTitle")), k3.key());
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
        let re = create_dir("./test");
        assert!(re.is_ok() || p.exists());
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
        assert_eq!(img.exif_data().unwrap().count(), Some(1));
        let k = ExifKey::try_from("Exif.Image.Artist").unwrap();
        let mut v = ExifValue::try_from(ExifTypeID::AsciiString).unwrap();
        v.read("L.H.B".as_bytes(), None).unwrap();
        img.exif_data_as_mut().unwrap().add(&k, &v).unwrap();
        assert_eq!(img.exif_data().unwrap().count(), Some(2));
        img.write_metadata().unwrap();
    }
}
