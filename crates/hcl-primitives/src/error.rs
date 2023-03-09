use alloc::string::{String, ToString};
use core::fmt;

/// The error type used by this crate.
#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub(crate) fn new<T: fmt::Display>(msg: T) -> Error {
        Error(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::new(msg)
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::new(msg)
    }
}
