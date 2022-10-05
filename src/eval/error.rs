use super::*;
use std::fmt;

/// The result type used by this module.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type returned by all fallible operations within this module.
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorKind>,
}

impl Error {
    pub(super) fn new(inner: ErrorKind) -> Error {
        Error {
            inner: Box::new(inner),
        }
    }

    /// Returns a reference to the `ErrorKind` for further error matching.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner
    }

    pub(super) fn unexpected<T>(value: T, expected: &'static str) -> Error
    where
        T: Into<Value>,
    {
        Error::new(ErrorKind::Unexpected(value.into(), expected))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        From::from(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::new(ErrorKind::Message(msg))
    }
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        From::from(err.to_string())
    }
}

impl std::error::Error for Error {}

/// An enum representing all kinds of errors that can happen during the evaluation of HCL
/// expressions and templates.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    RawExpression,
    Message(String),
    UndefinedVariable(Identifier),
    UndefinedFunc(Identifier),
    Unexpected(Value, &'static str),
    IndexOutOfBounds(usize),
    InvalidUnaryOp(UnaryOperator, Value),
    InvalidBinaryOp(Value, BinaryOperator, Value),
    NoSuchKey(String),
    KeyAlreadyExists(String),
    FuncCall(Identifier, String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::RawExpression => f.write_str("raw expressions cannot be evaluated"),
            ErrorKind::Message(msg) => f.write_str(msg),
            ErrorKind::UndefinedVariable(ident) => {
                write!(f, "undefined variable `{}`", ident)
            }
            ErrorKind::UndefinedFunc(ident) => {
                write!(f, "undefined function `{}`", ident)
            }
            ErrorKind::Unexpected(value, expected) => {
                write!(f, "unexpected value `{}`, expected {}", value, expected)
            }
            ErrorKind::IndexOutOfBounds(index) => write!(f, "index out of bounds: {}", index),
            ErrorKind::NoSuchKey(key) => write!(f, "no such key: `{}`", key),
            ErrorKind::KeyAlreadyExists(key) => write!(f, "key `{}` already exists", key),
            ErrorKind::InvalidUnaryOp(operator, value) => write!(
                f,
                "unary operator `{}` is not applicable to `{}`",
                operator, value,
            ),
            ErrorKind::InvalidBinaryOp(lhs, operator, rhs) => write!(
                f,
                "binary operator `{}` is not applicable to `{}` and `{}`",
                operator, lhs, rhs
            ),
            ErrorKind::FuncCall(name, msg) => {
                write!(f, "invalid call to function `{}`: {}", name, msg)
            }
        }
    }
}
