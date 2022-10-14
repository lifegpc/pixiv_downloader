#[cfg(feature = "server")]
use crate::server::result::JSONError;

/// Try with custom error message
pub trait TryErr2<T, E> {
    /// try with custom error message
    fn try_err2(&self, err: E) -> Result<T, E>;
}

/// Try with custom error message
pub trait TryErr<T, E> {
    /// try with custom error message
    fn try_err(self, err: E) -> Result<T, E>;
}

#[cfg(feature = "server")]
/// A quick way to return detailed JSON error
pub trait TryErr3<T> {
    /// A quick way to return detailed JSON error
    /// * `code` - error code
    /// * `msg` - error message
    fn try_err3<S: AsRef<str> + ?Sized>(self, code: i32, msg: &S) -> Result<T, JSONError>;
}

impl<T: ToOwned + ToOwned<Owned = T>, E> TryErr2<T, E> for Option<T> {
    fn try_err2(&self, v: E) -> Result<T, E> {
        match self {
            Some(r) => Ok(r.to_owned()),
            None => Err(v),
        }
    }
}

impl<T, E> TryErr<T, E> for Option<T> {
    fn try_err(self, err: E) -> Result<T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err),
        }
    }
}

impl<T, E, E2> TryErr<T, E> for Result<T, E2> {
    fn try_err(self, err: E) -> Result<T, E> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(err),
        }
    }
}

impl<E> TryErr<(), E> for bool {
    fn try_err(self, err: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[cfg(feature = "server")]
impl<T, E> TryErr3<T> for Result<T, E>
where
    E: std::fmt::Debug + std::fmt::Display,
{
    fn try_err3<S: AsRef<str> + ?Sized>(self, code: i32, msg: &S) -> Result<T, JSONError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(JSONError::from((
                code,
                format!("{} {}", msg.as_ref(), e),
                format!("{:?}", e),
            ))),
        }
    }
}

#[cfg(feature = "server")]
impl<T> TryErr3<T> for Option<T> {
    fn try_err3<S: AsRef<str> + ?Sized>(self, code: i32, msg: &S) -> Result<T, JSONError> {
        match self {
            Some(v) => Ok(v),
            None => Err(JSONError::from((code, msg.as_ref().to_string(), None))),
        }
    }
}
