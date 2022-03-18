/// Return raw pointer of the handle
pub trait ToRawHandle<T> {
    /// Return raw pointer of the handle
    unsafe fn to_raw_handle(&self) -> *mut T;
}
