use flagset::Flags;
use flagset::FlagSet;

pub trait ToFlagSet<T: Flags> {
    fn to_flag_set(&self) -> FlagSet<T>;
    fn to_bits(&self) -> T::Type {
        self.to_flag_set().bits()
    }
}

impl<T: Flags> ToFlagSet<T> for T where FlagSet<T>: From<T> {
    fn to_flag_set(&self) -> FlagSet<T> {
        FlagSet::from(self.clone())
    }
}

impl<T: Flags> ToFlagSet<T> for FlagSet<T> {
    fn to_flag_set(&self) -> FlagSet<T> {
        self.clone()
    }
}

impl<T: Flags> ToFlagSet<T> for Option<T> where FlagSet<T>: From<T> {
    fn to_flag_set(&self) -> FlagSet<T> {
        match self {
            Some(s) => { FlagSet::from(s.clone()) }
            None => { FlagSet::default() }
        }
    }
}
