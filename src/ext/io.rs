use proc_macros::define_struct_reader_fn;
use proc_macros::define_struct_writer_fn;
use proc_macros::impl_struct_reader_read;
use proc_macros::impl_struct_writer_write;
use std::io::Read;
use std::io::Write;

/// Read number.
pub trait StructRead {
    /// The error type
    type Error;
    define_struct_reader_fn!(u8);
    define_struct_reader_fn!(i8);
    define_struct_reader_fn!(u16);
    define_struct_reader_fn!(i16);
    define_struct_reader_fn!(u32);
    define_struct_reader_fn!(i32);
    define_struct_reader_fn!(u64);
    define_struct_reader_fn!(i64);
    define_struct_reader_fn!(usize);
    define_struct_reader_fn!(isize);
    define_struct_reader_fn!(u128);
    define_struct_reader_fn!(i128);
    /// Read exact number of bytes.
    /// * `size` - The number of bytes
    /// 
    /// Returns io error or the bytes.
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, Self::Error>;
}

impl<T: Read> StructRead for T {
    type Error = std::io::Error;
    impl_struct_reader_read!(u8);
    impl_struct_reader_read!(i8);
    impl_struct_reader_read!(u16);
    impl_struct_reader_read!(i16);
    impl_struct_reader_read!(u32);
    impl_struct_reader_read!(i32);
    impl_struct_reader_read!(u64);
    impl_struct_reader_read!(i64);
    impl_struct_reader_read!(usize);
    impl_struct_reader_read!(isize);
    impl_struct_reader_read!(u128);
    impl_struct_reader_read!(i128);

    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, Self::Error> {
        let mut h = self.take(size as u64);
        let mut r = Vec::new();
        let s = h.read_to_end(&mut r)?;
        if s != size {
            Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
        } else {
            Ok(r)
        }
    }
}

/// Clear all datas in file
pub trait ClearFile {
    /// Clear all datas in file
    fn clear_file(&mut self) -> std::io::Result<()>;
}

/// Write number.
pub trait StructWrite {
    /// The error type
    type Error;
    define_struct_writer_fn!(u8);
    define_struct_writer_fn!(i8);
    define_struct_writer_fn!(u16);
    define_struct_writer_fn!(i16);
    define_struct_writer_fn!(u32);
    define_struct_writer_fn!(i32);
    define_struct_writer_fn!(u64);
    define_struct_writer_fn!(i64);
    define_struct_writer_fn!(usize);
    define_struct_writer_fn!(isize);
    define_struct_writer_fn!(u128);
    define_struct_writer_fn!(i128);
}

impl<T: Write> StructWrite for T {
    type Error = std::io::Error;
    impl_struct_writer_write!(u8);
    impl_struct_writer_write!(i8);
    impl_struct_writer_write!(u16);
    impl_struct_writer_write!(i16);
    impl_struct_writer_write!(u32);
    impl_struct_writer_write!(i32);
    impl_struct_writer_write!(u64);
    impl_struct_writer_write!(i64);
    impl_struct_writer_write!(usize);
    impl_struct_writer_write!(isize);
    impl_struct_writer_write!(u128);
    impl_struct_writer_write!(i128);
}
