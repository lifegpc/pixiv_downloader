/// Try with custom error message
pub trait TryErr<T, E> {
    /// try with custom error message
    fn try_err(&self, err: E) -> Result<T, E>;
}

impl<T: ToOwned + ToOwned<Owned = T>, E> TryErr<T, E> for Option<T> {
    fn try_err(&self, v: E) -> Result<T, E> {
        match self {
            Some(r) => { Ok(r.to_owned()) }
            None => { Err(v) }
        }
    }
}
