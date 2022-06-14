use spin_on::spin_on;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::time::Duration;

pub trait GetRwLock {
    type Target;
    fn get_ref(&self) -> RwLockReadGuard<Self::Target>;
    fn get_mut(&self) -> RwLockWriteGuard<Self::Target>;
}

impl<T: Sized> GetRwLock for RwLock<T> {
    type Target = T;
    fn get_ref<'a>(&'a self) -> RwLockReadGuard<'a, Self::Target> {
        loop {
            match self.try_read() {
                Ok(f) => {
                    return f;
                }
                Err(_) => {
                    spin_on(tokio::time::sleep(Duration::new(0, 1_000_000)));
                }
            }
        }
    }
    fn get_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Self::Target> {
        loop {
            match self.try_write() {
                Ok(f) => {
                    return f;
                }
                Err(_) => {
                    spin_on(tokio::time::sleep(Duration::new(0, 1_000_000)));
                }
            }
        }
    }
}
