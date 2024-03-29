use crate::_exif;
pub use crate::_exif::{ExifDataRef, ExifDatumRef, ExifValueRef};
use crate::ext::rawhandle::AsNonNullPtr;
use crate::ext::rawhandle::FromRawHandle;
use crate::ext::rawhandle::ToRawHandle;
use c_fixed_string::CFixedStr;
use int_enum::IntEnum;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::borrow::ToOwned;
use std::clone::Clone;
use std::ffi::CStr;
use std::ffi::CString;
use std::ffi::OsStr;
#[cfg(test)]
use std::fs::copy;
#[cfg(test)]
use std::fs::create_dir;
use std::iter::{DoubleEndedIterator, ExactSizeIterator, Iterator};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Drop;
use std::os::raw::c_long;
use std::path::Path;
use std::ptr::NonNull;

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
    key: NonNull<_exif::ExifKey>,
}

impl TryFrom<CString> for ExifKey {
    type Error = ();
    fn try_from(value: CString) -> Result<Self, Self::Error> {
        let ptr = value.as_ptr();
        let re = unsafe { _exif::exif_create_key_by_key(ptr) };
        Ok(Self {
            key: NonNull::new(re).ok_or(())?,
        })
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
        unsafe { _exif::exif_free_key(self.key.as_ptr()) };
    }
}

impl Clone for ExifKey {
    fn clone(&self) -> Self {
        let key = unsafe { _exif::exif_create_key_by_another(self.key.as_ptr()) };
        Self {
            key: NonNull::new(key).expect("Out of memory:"),
        }
    }
}

impl AsNonNullPtr<_exif::ExifKey> for ExifKey {
    fn as_non_null(&self) -> &NonNull<_exif::ExifKey> {
        &self.key
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
        Ok(Self {
            key: NonNull::new(re).ok_or(())?,
        })
    }

    /// Return the key of the metadatum as a string.
    /// The key is of the form `familyName.groupName.tagName`
    /// # Note
    /// However that the key is not necessarily unique, e.g., an ExifData may contain multiple metadata with the same key.
    pub fn key(&self) -> String {
        let r = unsafe { _exif::exif_get_key_key(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return an identifier for the type of metadata (the first part of the key)
    pub fn family_name(&self) -> String {
        let r = unsafe { _exif::exif_get_key_family_name(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return the name of the group (the second part of the key)
    pub fn group_name(&self) -> String {
        let r = unsafe { _exif::exif_get_key_group_name(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return the name of the tag (which is also the third part of the key)
    pub fn tag_name(&self) -> String {
        let r = unsafe { _exif::exif_get_key_tag_name(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return the tag number.
    pub fn tag(&self) -> u16 {
        unsafe { _exif::exif_get_key_tag_tag(self.key.as_ptr()) }
    }

    /// Return a label for the tag.
    pub fn tag_label(&self) -> String {
        let r = unsafe { _exif::exif_get_key_tag_label(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return the tag description.
    pub fn tag_desc(&self) -> String {
        let r = unsafe { _exif::exif_get_key_tag_desc(self.key.as_ptr()) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CStr::from_ptr(r) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        s.to_string_lossy().into_owned()
    }

    /// Return the default type id for this tag.
    pub fn default_typeid(&self) -> Option<ExifTypeID> {
        let r = unsafe { _exif::exif_get_key_default_type_id(self.key.as_ptr()) };
        ExifTypeID::from_int(r).ok()
    }
}

/// Common interface for all types of values used with metadata.
pub struct ExifValue {
    value: NonNull<_exif::ExifValue>,
}

impl ExifValue {
    pub unsafe fn from_raw_pointer(value: *mut _exif::ExifValue) -> Self {
        Self {
            value: unsafe { NonNull::new_unchecked(value) },
        }
    }
}

impl TryFrom<ExifTypeID> for ExifValue {
    type Error = ();
    fn try_from(value: ExifTypeID) -> Result<Self, Self::Error> {
        let d = value.int_value();
        let r = unsafe { _exif::exif_create_value(d) };
        Ok(Self {
            value: NonNull::new(r).ok_or(())?,
        })
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

impl Borrow<ExifValueRef> for ExifValue {
    fn borrow(&self) -> &ExifValueRef {
        self.deref()
    }
}

impl BorrowMut<ExifValueRef> for ExifValue {
    fn borrow_mut(&mut self) -> &mut ExifValueRef {
        self.deref_mut()
    }
}

impl Deref for ExifValue {
    type Target = ExifValueRef;
    fn deref(&self) -> &Self::Target {
        unsafe {
            ExifValueRef::from_const_handle(
                _exif::exif_value_get_ref(self.as_ptr()) as *const ExifValueRef
            )
        }
    }
}

impl DerefMut for ExifValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { ExifValueRef::from_raw_handle(_exif::exif_value_get_ref(self.as_ptr())) }
    }
}

impl Drop for ExifValue {
    fn drop(&mut self) {
        unsafe { _exif::exif_free_value(self.value.as_ptr()) };
    }
}

impl AsNonNullPtr<_exif::ExifValue> for ExifValue {
    fn as_non_null(&self) -> &NonNull<_exif::ExifValue> {
        &self.value
    }
}

#[allow(dead_code)]
impl ExifValueRef {
    /// Return the type identifier (Exif data format type).
    pub fn type_id(&self) -> Option<ExifTypeID> {
        let value = unsafe { self.to_raw_handle() };
        let r = unsafe { _exif::exif_get_value_type_id(value) };
        let re = ExifTypeID::from_int(r);
        if re.is_err() {
            return None;
        }
        Some(re.unwrap())
    }

    /// Return the number of components of the value.
    pub fn count(&self) -> usize {
        let value = unsafe { self.to_raw_handle() };
        unsafe { _exif::exif_get_value_count(value) }
    }

    /// Return the size of the value in bytes.
    pub fn size(&self) -> usize {
        let value = unsafe { self.to_raw_handle() };
        unsafe { _exif::exif_get_value_size(value) }
    }

    /// Return the size of the data area, 0 if there is none.
    pub fn size_data_area(&self) -> usize {
        let value = unsafe { self.to_raw_handle() };
        unsafe { _exif::exif_get_value_size_data_area(value) }
    }

    /// Read the value from a character buffer.
    /// * `buf` - Buffer
    /// * `byte_order` - Applicable byte order (little or big endian). Default: invaild.
    pub fn read(&mut self, buf: &[u8], byte_order: Option<ExifByteOrder>) -> Result<(), ()> {
        let value = unsafe { self.to_raw_handle() };
        let buf_len = buf.len() as c_long;
        let order = match byte_order {
            Some(o) => o,
            None => ExifByteOrder::Invalid,
        };
        let order = order.int_value();
        let ptr = buf.as_ptr();
        let r = unsafe { _exif::exif_value_read(value, ptr, buf_len, order) };
        if r == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Return the value as a string
    pub fn to_string(&self) -> Result<CString, ()> {
        let value = unsafe { self.to_raw_handle() };
        let mut size: Vec<usize> = vec![0];
        let ptr = size.as_mut_ptr();
        let r = unsafe { _exif::exif_value_to_string(value, ptr) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CFixedStr::from_mut_ptr(r, size[0]) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        if !self.ok() {
            return Err(());
        }
        Ok(s.into_c_string())
    }

    /// Return the n-th component of the value as a string.
    pub fn to_nth_string(&self, n: usize) -> Result<Option<CString>, ()> {
        let value = unsafe { self.to_raw_handle() };
        let count = self.count();
        if n >= count {
            return Ok(None);
        }
        let mut size: Vec<usize> = vec![0];
        let ptr = size.as_mut_ptr();
        let r = unsafe { _exif::exif_value_to_string2(value, ptr, n) };
        if r.is_null() {
            panic!("Out of memory.");
        }
        let s = unsafe { CFixedStr::from_mut_ptr(r, size[0]) };
        let s = s.to_owned();
        unsafe { _exif::exif_free(r as *mut ::std::os::raw::c_void) };
        if !self.ok() {
            return Err(());
        }
        Ok(Some(s.into_c_string()))
    }

    /// Check the ok status indicator.
    /// After a `to<Type>` conversion, this indicator shows whether the conversion was successful.
    pub fn ok(&self) -> bool {
        let value = unsafe { self.to_raw_handle() };
        let r = unsafe { _exif::exif_get_value_ok(value) };
        r != 0
    }

    /// Convert the n-th component of the value to a int64
    pub fn to_int64(&self, n: usize) -> Result<Option<i64>, ()> {
        let value = unsafe { self.to_raw_handle() };
        let c = self.count();
        if n >= c {
            return Ok(None);
        }
        let r = unsafe { _exif::exif_value_to_int64(value, n) };
        if !self.ok() {
            return Err(());
        }
        Ok(Some(r))
    }
}

impl ToRawHandle<ExifValueRef> for ExifValueRef {
    unsafe fn to_raw_handle(&self) -> *mut ExifValueRef {
        self.to_const_handle() as *mut ExifValueRef
    }
    unsafe fn to_const_handle(&self) -> *const ExifValueRef {
        self
    }
}

impl ToOwned for ExifValueRef {
    type Owned = ExifValue;
    fn to_owned(&self) -> Self::Owned {
        let v = unsafe { self.to_raw_handle() };
        if v.is_null() {
            panic!("ExifValue reference is null.");
        }
        let r = unsafe { _exif::exif_value_ref_clone(v) };
        if r.is_null() {
            panic!("Failed to convert ExifValueRef to ExifValue");
        }
        unsafe { ExifValue::from_raw_pointer(r) }
    }
}

#[allow(dead_code)]
impl ExifDatumRef {
    /// Return the key of the Exifdatum.
    pub fn key(&self) -> Option<String> {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_datum_key(data) };
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

    /// Set the value.
    pub fn set_value<'a>(&'a mut self, value: &ExifValueRef) {
        let data = unsafe { self.to_raw_handle() };
        let v = unsafe { value.to_raw_handle() };
        if data.is_null() || v.is_null() {
            return;
        }
        unsafe { _exif::exif_datum_set_value(data, v) }
    }

    /// Return a constant reference to the value.
    ///
    /// This method is provided mostly for convenient and versatile output of the value which can (to some extent) be formatted through standard stream manipulators.
    /// An Error is thrown if the value is not set; as an alternative to catching it, one can use count() to check if there is any data before calling this method.
    pub fn value<'a>(&'a self) -> Option<&'a ExifValueRef> {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_datum_value(data) };
        if r.is_null() {
            return None;
        }
        Some(unsafe { ExifValueRef::from_const_handle(r as *const ExifValueRef) })
    }
}

impl ToRawHandle<ExifDatumRef> for ExifDatumRef {
    unsafe fn to_raw_handle(&self) -> *mut ExifDatumRef {
        self.to_const_handle() as *mut ExifDatumRef
    }
    unsafe fn to_const_handle(&self) -> *const ExifDatumRef {
        self
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
    pub fn add(&mut self, key: &ExifKey, value: &ExifValueRef) -> Result<(), ()> {
        let data = unsafe { self.to_raw_handle() };
        let k = key.as_ptr();
        let v = unsafe { value.to_raw_handle() };
        if v.is_null() || data.is_null() {
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

    pub fn iter<'a>(&'a self) -> Option<ExifDataItor<'a>> {
        let data = unsafe { self.to_raw_handle() };
        let count = match self.count() {
            Some(count) => count,
            None => {
                return None;
            }
        };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_ref_iter(data) };
        if r.is_null() {
            return None;
        }
        Some(ExifDataItor {
            itor: r,
            count,
            phantom: PhantomData,
        })
    }

    pub fn iter_mut<'a>(&'a mut self) -> Option<ExifDataMutItor<'a>> {
        let data = unsafe { self.to_raw_handle() };
        let count = match self.count() {
            Some(count) => count,
            None => {
                return None;
            }
        };
        if data.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_ref_iter_mut(data) };
        if r.is_null() {
            return None;
        }
        Some(ExifDataMutItor {
            itor: r,
            count,
            phantom: PhantomData,
        })
    }

    pub fn sort_by_key(&mut self) {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return;
        }
        unsafe { _exif::exif_data_ref_sort_by_key(data) }
    }

    pub fn sort_by_tag(&mut self) {
        let data = unsafe { self.to_raw_handle() };
        if data.is_null() {
            return;
        }
        unsafe { _exif::exif_data_ref_sort_by_tag(data) }
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

pub struct ExifDataItor<'a> {
    itor: *mut _exif::ExifDataItor,
    count: usize,
    phantom: PhantomData<&'a _exif::ExifDataItor>,
}

impl<'a> Drop for ExifDataItor<'a> {
    fn drop(&mut self) {
        if !self.itor.is_null() {
            unsafe { _exif::exif_free_data_itor(self.itor) };
            self.itor = std::ptr::null_mut();
        }
    }
}

impl<'a> Iterator for ExifDataItor<'a> {
    type Item = &'a ExifDatumRef;
    fn next(&mut self) -> Option<Self::Item> {
        if self.itor.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_itor_next(self.itor) };
        if r.is_null() {
            return None;
        }
        Some(unsafe { ExifDatumRef::from_const_handle(r as *const ExifDatumRef) })
    }
}

impl<'a> DoubleEndedIterator for ExifDataItor<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.itor.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_itor_next_back(self.itor) };
        if r.is_null() {
            return None;
        }
        Some(unsafe { ExifDatumRef::from_const_handle(r as *const ExifDatumRef) })
    }
}

impl<'a> ExactSizeIterator for ExifDataItor<'a> {
    fn len(&self) -> usize {
        self.count
    }
}

pub struct ExifDataMutItor<'a> {
    itor: *mut _exif::ExifDataMutItor,
    count: usize,
    phantom: PhantomData<&'a mut _exif::ExifDataMutItor>,
}

impl<'a> Drop for ExifDataMutItor<'a> {
    fn drop(&mut self) {
        if !self.itor.is_null() {
            unsafe { _exif::exif_free_data_mutitor(self.itor) };
            self.itor = std::ptr::null_mut();
        }
    }
}

impl<'a> Iterator for ExifDataMutItor<'a> {
    type Item = &'a mut ExifDatumRef;
    fn next(&mut self) -> Option<Self::Item> {
        if self.itor.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_mutitor_next(self.itor) };
        if r.is_null() {
            return None;
        }
        Some(unsafe { ExifDatumRef::from_raw_handle(r) })
    }
}

impl<'a> DoubleEndedIterator for ExifDataMutItor<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.itor.is_null() {
            return None;
        }
        let r = unsafe { _exif::exif_data_mutitor_next_back(self.itor) };
        if r.is_null() {
            return None;
        }
        Some(unsafe { ExifDatumRef::from_raw_handle(r) })
    }
}

impl<'a> ExactSizeIterator for ExifDataMutItor<'a> {
    fn len(&self) -> usize {
        self.count
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
    pub fn exif_data<'a>(&'a self) -> Option<&'a ExifDataRef> {
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
    pub fn exif_data_as_mut<'a>(&'a mut self) -> Option<&'a mut ExifDataRef> {
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
    assert_eq!(String::from("Exif.Image.XPTitle"), k.key());
    assert_eq!(String::from("Exif"), k.family_name());
    assert_eq!(String::from("Image"), k.group_name());
    assert_eq!(String::from("XPTitle"), k.tag_name());
    assert_eq!(40091, k.tag());
    assert_eq!(String::from("Windows Title"), k.tag_label());
    assert_eq!(
        String::from("Title tag used by Windows, encoded in UCS2"),
        k.tag_desc()
    );
    assert_eq!(Some(ExifTypeID::BYTE), k.default_typeid());
    let k2 = ExifKey::from_id(40091, "Image");
    assert!(k2.is_ok());
    let k2 = k2.unwrap();
    assert_eq!(String::from("Exif.Image.XPTitle"), k2.key());
    let k3 = k2.clone();
    assert_eq!(String::from("Exif.Image.XPTitle"), k3.key());
}

#[test]
fn test_exif_value() {
    let v = ExifValue::try_from(ExifTypeID::BYTE);
    assert!(v.is_ok());
    let mut v = v.unwrap();
    assert_eq!(Some(ExifTypeID::BYTE), v.type_id());
    assert_eq!(0, v.count());
    assert_eq!(0, v.size());
    assert_eq!(0, v.size_data_area());
    assert!(v.read("test".as_bytes(), None).is_ok());
    assert_eq!(4, v.count());
    assert_eq!(4, v.size());
    let c = CString::new("116 101 115 116").unwrap();
    assert_eq!(Ok(c), v.to_string());
    assert_eq!(4, v.count());
    assert_eq!(4, v.size());
    assert_eq!(Ok(Some(CString::new("116").unwrap())), v.to_nth_string(0));
    assert_eq!(Ok(None), v.to_nth_string(4));
    assert_eq!(Ok(None), v.to_int64(4));
    assert_eq!(Ok(Some(116)), v.to_int64(0));
    let mut v2 = ExifValue::try_from(ExifTypeID::SLongLong).unwrap();
    v2.read(&(102345i64).to_le_bytes(), Some(ExifByteOrder::Little))
        .unwrap();
    assert_eq!(8, v2.count());
    let v3: &ExifValueRef = &v2;
    assert_eq!(
        v3.to_string(),
        Ok(CString::new("201 143 1 0 0 0 0 0").unwrap())
    );
    let mut v4 = v3.to_owned();
    v4.read(&(102346i64).to_le_bytes(), Some(ExifByteOrder::Little))
        .unwrap();
    assert_eq!(
        v3.to_string(),
        Ok(CString::new("201 143 1 0 0 0 0 0").unwrap())
    );
    assert_eq!(
        v4.to_string(),
        Ok(CString::new("202 143 1 0 0 0 0 0").unwrap())
    );
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
    d.add(&k, &v).unwrap();
    let k2 = ExifKey::try_from("Exif.Image.PageName").unwrap();
    let mut v2 = ExifValue::try_from(ExifTypeID::AsciiString).unwrap();
    v2.read("p1".as_bytes(), None).unwrap();
    d.add(&k2, &v2).unwrap();
    assert_eq!(Some(2), d.count());
    {
        let mut i = d.iter().unwrap();
        let f = i.next().unwrap();
        assert_eq!(f.key(), Some(String::from("Exif.Image.XPTitle")));
        assert_eq!(
            i.next().unwrap().key(),
            Some(String::from("Exif.Image.PageName"))
        );
        assert!(i.next().is_none());
        let mut i = 0;
        for data in d.iter().unwrap() {
            i += 1;
            match data.key().unwrap().as_str() {
                "Exif.Image.PageName" => {
                    assert_eq!(i, 2);
                    let v = data.value().unwrap();
                    assert_eq!(v.to_string(), Ok(CString::new("p1").unwrap()));
                }
                "Exif.Image.XPTitle" => assert_eq!(i, 1),
                _ => {}
            }
        }
        assert_eq!(i, 2);
        let mut i = d.iter().unwrap();
        let f = i.next().unwrap();
        let f2 = i.next().unwrap();
        assert_eq!(f.key(), Some(String::from("Exif.Image.XPTitle")));
        assert_eq!(f2.key(), Some(String::from("Exif.Image.PageName")));
        let mut i = d.iter().unwrap();
        assert_eq!(
            i.next_back().unwrap().key(),
            Some(String::from("Exif.Image.PageName"))
        );
        assert_eq!(
            i.next().unwrap().key(),
            Some(String::from("Exif.Image.XPTitle"))
        );
        assert!(i.next().is_none());
        assert_eq!(
            d.iter().unwrap().position(|d| match d.key() {
                Some(key) => {
                    match key.as_str() {
                        "Exif.Image.PageName" => true,
                        _ => false,
                    }
                }
                None => false,
            }),
            Some(1)
        );
        assert_eq!(
            d.iter().unwrap().rposition(|d| match d.key() {
                Some(key) => {
                    match key.as_str() {
                        "Exif.Image.XPTitle" => true,
                        _ => false,
                    }
                }
                None => false,
            }),
            Some(0)
        );
    }
    for i in d.iter_mut().unwrap() {
        match i.key() {
            Some(key) => match key.as_str() {
                "Exif.Image.PageName" => {
                    let mut v = ExifValue::try_from(ExifTypeID::AsciiString).unwrap();
                    v.read("p2".as_bytes(), None).unwrap();
                    i.set_value(&v);
                }
                _ => {}
            },
            _ => {}
        }
    }
    let p = d
        .iter()
        .unwrap()
        .nth(1)
        .unwrap()
        .value()
        .unwrap()
        .to_string();
    assert_eq!(p, Ok(CString::new("p2").unwrap()));
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
