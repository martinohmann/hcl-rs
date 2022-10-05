use super::*;
use std::fmt;

/// The result type used by this module.
pub type EvalResult<T> = Result<T, EvalError>;

/// The error type returned by all fallible operations within this module.
#[derive(Debug)]
pub struct EvalError {
    inner: Box<EvalErrorKind>,
}

impl EvalError {
    pub(super) fn new(inner: EvalErrorKind) -> EvalError {
        EvalError {
            inner: Box::new(inner),
        }
    }

    /// Returns a reference to the `EvalErrorKind` for further error matching.
    pub fn kind(&self) -> &EvalErrorKind {
        &self.inner
    }

    pub(super) fn unexpected<T>(value: T, expected: &'static str) -> EvalError
    where
        T: Into<Value>,
    {
        EvalError::new(EvalErrorKind::Unexpected(value.into(), expected))
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<&str> for EvalError {
    fn from(msg: &str) -> Self {
        From::from(msg.to_string())
    }
}

impl From<String> for EvalError {
    fn from(msg: String) -> Self {
        EvalError::new(EvalErrorKind::Message(msg))
    }
}

impl From<Error> for EvalError {
    fn from(err: Error) -> Self {
        From::from(err.to_string())
    }
}

impl std::error::Error for EvalError {}

/// An enum representing all kinds of errors that can happen during the evaluation of HCL
/// expressions and templates.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EvalErrorKind {
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

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::RawExpression => f.write_str("raw expressions cannot be evaluated"),
            EvalErrorKind::Message(msg) => f.write_str(msg),
            EvalErrorKind::UndefinedVariable(ident) => {
                write!(f, "undefined variable `{}`", ident)
            }
            EvalErrorKind::UndefinedFunc(ident) => {
                write!(f, "undefined function `{}`", ident)
            }
            EvalErrorKind::Unexpected(value, expected) => {
                write!(f, "unexpected value `{}`, expected {}", value, expected)
            }
            EvalErrorKind::IndexOutOfBounds(index) => write!(f, "index out of bounds: {}", index),
            EvalErrorKind::NoSuchKey(key) => write!(f, "no such key: `{}`", key),
            EvalErrorKind::KeyAlreadyExists(key) => write!(f, "key `{}` already exists", key),
            EvalErrorKind::InvalidUnaryOp(operator, value) => write!(
                f,
                "unary operator `{}` is not applicable to `{}`",
                operator, value,
            ),
            EvalErrorKind::InvalidBinaryOp(lhs, operator, rhs) => write!(
                f,
                "binary operator `{}` is not applicable to `{}` and `{}`",
                operator, lhs, rhs
            ),
            EvalErrorKind::FuncCall(name, msg) => {
                write!(f, "invalid call to function `{}`: {}", name, msg)
            }
        }
    }
}
