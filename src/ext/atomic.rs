use std::sync::atomic::Ordering;

/// A trait to help to load and store atomic value quickly.
pub trait AtomicQuick<T> {
    /// Loads a value from the atomic integer
    fn qload(&self) -> T;
    /// Stores a value into the atomic integer.
    fn qstore(&self, value: T);
    #[inline]
    /// Stores a value into the atomic integer.
    /// Alias for [Self::qstore]
    fn qsave(&self, value: T) {
        self.qstore(value)
    }
}

macro_rules! impl_atomic_quick_with_atomic {
    ($type1:ty, $type2:ty) => {
        impl AtomicQuick<$type2> for $type1 {
            #[inline]
            fn qload(&self) -> $type2 {
                self.load(Ordering::Relaxed)
            }
            #[inline]
            fn qstore(&self, value: $type2) {
                self.store(value, Ordering::Relaxed)
            }
        }
    }
}

impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicBool, bool);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicI8, i8);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicU8, u8);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicI16, i16);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicU16, u16);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicI32, i32);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicU32, u32);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicI64, i64);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicU64, u64);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicIsize, isize);
impl_atomic_quick_with_atomic!(std::sync::atomic::AtomicUsize, usize);
