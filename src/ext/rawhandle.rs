use std::ptr::NonNull;

/// Return raw pointer of the handle
pub trait ToRawHandle<T> {
    /// Return raw pointer of the handle
    unsafe fn to_raw_handle(&self) -> *mut T;

    /// Return the const raw pointer of the handle
    unsafe fn to_const_handle(&self) -> *const T {
        self.to_raw_handle() as *const T
    }
}

/// Return a Non null Pointer
pub trait AsNonNullPtr<T> {
    /// Return a [NonNull]
    fn as_non_null(&self) -> &NonNull<T>;
    /// Return a non null pointer
    fn as_ptr(&self) -> *mut T {
        self.as_non_null().as_ptr()
    }
}

pub trait FromRawHandle<T> {
    unsafe fn from_raw_handle<'a>(ptr: *mut T) -> &'a mut Self;
    unsafe fn from_const_handle<'a>(ptr: *const T) -> &'a Self;
}

impl<T> FromRawHandle<Self> for T {
    unsafe fn from_raw_handle<'a>(ptr: *mut T) -> &'a mut Self {
        &mut *(ptr)
    }
    unsafe fn from_const_handle<'a>(ptr: *const Self) -> &'a Self {
        &*(ptr)
    }
}
