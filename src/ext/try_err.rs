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
