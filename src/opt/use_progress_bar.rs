use crate::ext::use_or_not::ToBool;
use crate::ext::use_or_not::UseOrNot;
use is_terminal::IsTerminal;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum StreamType {
    Stdout,
    Stderr,
    Stdin,
}

impl StreamType {
    pub fn is_terminal(&self) -> bool {
        match self {
            Self::Stdout => std::io::stdout().is_terminal(),
            Self::Stderr => std::io::stderr().is_terminal(),
            Self::Stdin => std::io::stdin().is_terminal(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UseProgressBar {
    v: UseOrNot,
    stream: StreamType,
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
            stream: StreamType::Stdout,
        }
    }
}

impl From<bool> for UseProgressBar {
    fn from(v: bool) -> Self {
        Self {
            v: UseOrNot::from(v),
            stream: StreamType::Stdout,
        }
    }
}

impl From<UseOrNot> for UseProgressBar {
    fn from(v: UseOrNot) -> Self {
        Self {
            v,
            stream: StreamType::Stdout,
        }
    }
}

impl ToBool for UseProgressBar {
    fn detect(&self) -> bool {
        self.stream.is_terminal()
    }
}
