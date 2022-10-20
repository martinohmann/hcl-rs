use super::*;
use std::fmt;

/// The result type used by this module.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error type returned by all fallible operations within this module.
#[derive(Debug, Clone)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    pub(super) fn new<T>(kind: T) -> Error
    where
        T: Into<ErrorKind>,
    {
        Error::new_with_expr(kind, None)
    }

    pub(super) fn new_with_expr<T>(kind: T, expr: Option<Expression>) -> Error
    where
        T: Into<ErrorKind>,
    {
        Error {
            inner: Box::new(ErrorInner::new(kind.into(), expr)),
        }
    }

    pub(super) fn unexpected<T>(value: T, expected: &'static str) -> Error
    where
        T: Into<Value>,
    {
        Error::new(ErrorKind::Unexpected(value.into(), expected))
    }

    /// Returns a reference to the `ErrorKind` for further error matching.
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }

    /// Returns a reference to the `Expression` that caused the error, if there is one.
    pub fn expr(&self) -> Option<&Expression> {
        self.inner.expr.as_ref()
    }

    /// Consumes the `Error` and returns the `ErrorKind`.
    pub fn into_kind(self) -> ErrorKind {
        self.inner.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::new(kind)
    }
}

impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        Error::new(ErrorKind::Message(err.to_string()))
    }
}

impl std::error::Error for Error {}

// The inner type that holds the actual error data. This is a separate type because it gets boxed
// to keep the size of the `Error` struct small.
#[derive(Debug, Clone)]
struct ErrorInner {
    kind: ErrorKind,
    expr: Option<Expression>,
}

impl ErrorInner {
    fn new(kind: ErrorKind, expr: Option<Expression>) -> ErrorInner {
        ErrorInner { kind, expr }
    }
}

impl fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        if let Some(expr) = &self.expr {
            write!(f, " in expression `{}`", expr)?;
        }

        Ok(())
    }
}

/// An enum representing all kinds of errors that can happen during the evaluation of HCL
/// expressions and templates.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Message(String),
    UndefinedVariable(Identifier),
    UndefinedFunc(Identifier),
    Unexpected(Value, &'static str),
    IndexOutOfBounds(usize),
    ImpossibleUnaryOp(UnaryOperator, Value),
    ImpossibleBinaryOp(Value, BinaryOperator, Value),
    NoSuchKey(String),
    KeyAlreadyExists(String),
    FuncCall(Identifier, String),
}

impl From<Error> for ErrorKind {
    fn from(err: Error) -> Self {
        err.into_kind()
    }
}

impl From<&str> for ErrorKind {
    fn from(msg: &str) -> Self {
        ErrorKind::Message(msg.to_owned())
    }
}

impl From<String> for ErrorKind {
    fn from(msg: String) -> Self {
        ErrorKind::Message(msg)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            ErrorKind::ImpossibleUnaryOp(operator, value) => write!(
                f,
                "unary operator `{}` is not applicable to `{}`",
                operator, value,
            ),
            ErrorKind::ImpossibleBinaryOp(lhs, operator, rhs) => write!(
                f,
                "binary operator `{}` is not applicable to `{}` and `{}`",
                operator, lhs, rhs
            ),
            ErrorKind::FuncCall(name, msg) => {
                write!(f, "error calling function `{}`: {}", name, msg)
            }
        }
    }
}
