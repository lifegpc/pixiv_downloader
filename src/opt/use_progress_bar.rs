use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;

#[derive(Clone, Copy, Debug)]
pub struct UseProgressBar {
    v: UseOrNot,
    stream: atty::Stream,
}

impl AsRef<Self> for UseProgressBar {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<UseOrNot> for UseProgressBar {
    fn as_ref(&self) -> &UseOrNot {
        self.v.as_ref()
    }
}

impl Default for UseProgressBar {
    fn default() -> Self {
        Self {
            v: UseOrNot::Auto,
            stream: atty::Stream::Stdout,
        }
    }
}

impl From<bool> for UseProgressBar {
    fn from(v: bool) -> Self {
        Self {
            v: UseOrNot::from(v),
            stream: atty::Stream::Stdout,
        }
    }
}

impl From<atty::Stream> for UseProgressBar {
    fn from(stream: atty::Stream) -> Self {
        Self {
            v: UseOrNot::Auto,
            stream,
        }
    }
}

impl From<UseOrNot> for UseProgressBar {
    fn from(v: UseOrNot) -> Self {
        Self {
            v,
            stream: atty::Stream::Stdout,
        }
    }
}

impl ToBool for UseProgressBar {
    fn detect(&self) -> bool {
        atty::is(self.stream)
    }
}
