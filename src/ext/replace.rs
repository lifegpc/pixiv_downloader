use crate::ext::rw_lock::GetRwLock;
use std::ops::DerefMut;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

/// Replace current value with another value
pub trait ReplaceWith<T> {
    /// Replace current value with another value
    /// * `another` - another value
    ///
    /// Returns the old value.
    fn replace_with(&mut self, another: T) -> T;
}

/// Replace current value with another value
///
/// If you want to mutably borrows, please use [ReplaceWith] instead.
pub trait ReplaceWith2<T> {
    /// Replace current value with another value
    /// * `another` - another value
    ///
    /// Returns the old value.
    fn replace_with2(&self, another: T) -> T;
}

impl<T> ReplaceWith<T> for T {
    #[inline]
    fn replace_with(&mut self, another: T) -> T {
        std::mem::replace(self, another)
    }
}

impl<'a, T> ReplaceWith<T> for RwLockWriteGuard<'a, T> {
    #[inline]
    fn replace_with(&mut self, another: T) -> T {
        self.deref_mut().replace_with(another)
    }
}

impl<T> ReplaceWith<T> for RwLock<T> {
    #[inline]
    fn replace_with(&mut self, another: T) -> T {
        self.replace_with2(another)
    }
}

impl<T> ReplaceWith2<T> for RwLock<T> {
    #[inline]
    fn replace_with2(&self, another: T) -> T {
        self.get_mut().replace_with(another)
    }
}

macro_rules! impl_replace_with_atomic {
    ($type1:ty, $type2:ty) => {
        impl ReplaceWith<$type2> for $type1 {
            #[inline]
            fn replace_with(&mut self, another: $type2) -> $type2 {
                self.replace_with2(another)
            }
        }
        impl ReplaceWith2<$type2> for $type1 {
            fn replace_with2(&self, another: $type2) -> $type2 {
                let ori = self.load(std::sync::atomic::Ordering::Relaxed);
                self.store(another, std::sync::atomic::Ordering::Relaxed);
                ori
            }
        }
    };
}

impl_replace_with_atomic!(std::sync::atomic::AtomicBool, bool);
impl_replace_with_atomic!(std::sync::atomic::AtomicI8, i8);
impl_replace_with_atomic!(std::sync::atomic::AtomicU8, u8);
impl_replace_with_atomic!(std::sync::atomic::AtomicI16, i16);
impl_replace_with_atomic!(std::sync::atomic::AtomicU16, u16);
impl_replace_with_atomic!(std::sync::atomic::AtomicI32, i32);
impl_replace_with_atomic!(std::sync::atomic::AtomicU32, u32);
impl_replace_with_atomic!(std::sync::atomic::AtomicI64, i64);
impl_replace_with_atomic!(std::sync::atomic::AtomicU64, u64);
impl_replace_with_atomic!(std::sync::atomic::AtomicIsize, isize);
impl_replace_with_atomic!(std::sync::atomic::AtomicUsize, usize);
