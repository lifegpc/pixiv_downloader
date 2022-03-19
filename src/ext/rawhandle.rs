/// Return raw pointer of the handle
pub trait ToRawHandle<T> {
    /// Return raw pointer of the handle
    unsafe fn to_raw_handle(&self) -> *mut T;

    /// Return the const raw pointer of the handle
    unsafe fn to_const_handle(&self) -> *const T {
        self.to_raw_handle() as *const T
    }
}
