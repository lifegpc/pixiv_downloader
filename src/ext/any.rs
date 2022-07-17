use std::any::Any;

pub trait AsAny<T: Any + ?Sized> {
    fn as_any(&self) -> &T;
    fn as_any_mut(&mut self) -> &mut T;
}
