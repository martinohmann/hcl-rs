use std::fmt;

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

impl std::error::Error for Error {}
