use nom::error::{ContextError, FromExternalError, ParseError};
use nom::Offset;
use std::fmt;

/// The result type used by this module.
pub type ParseResult<T, E = Error> = std::result::Result<T, E>;

/// The result type used by parsers internally.
pub type IResult<I, O, E = InternalError<I>> = nom::IResult<I, O, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind<T = String> {
    Nom(nom::error::ErrorKind),
    Context(&'static str),
    Char(char),
    Tag(T),
}

impl<'a> ErrorKind<&'a str> {
    fn into_owned(self) -> ErrorKind<String> {
        match self {
            ErrorKind::Nom(kind) => ErrorKind::Nom(kind),
            ErrorKind::Context(ctx) => ErrorKind::Context(ctx),
            ErrorKind::Char(ch) => ErrorKind::Char(ch),
            ErrorKind::Tag(tag) => ErrorKind::Tag(tag.to_owned()),
        }
    }
}

impl<T> fmt::Display for ErrorKind<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Context(ctx) => write!(f, "expected {ctx}"),
            ErrorKind::Nom(kind) => write!(f, "error in {kind:?} parser"),
            ErrorKind::Char(ch) => write!(f, "expected char `{ch}`"),
            ErrorKind::Tag(tag) => write!(f, "expected `{tag}`"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InternalError<I> {
    pub input: I,
    pub kind: ErrorKind<I>,
}

impl<I> InternalError<I> {
    #[inline]
    pub(super) fn new(input: I, kind: ErrorKind<I>) -> InternalError<I> {
        InternalError { input, kind }
    }
}

impl<I> ParseError<I> for InternalError<I> {
    #[inline]
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        InternalError::new(input, ErrorKind::Nom(kind))
    }

    #[inline]
    fn from_char(input: I, ch: char) -> Self {
        InternalError::new(input, ErrorKind::Char(ch))
    }

    #[inline]
    fn append(_: I, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> ContextError<I> for InternalError<I> {
    #[inline]
    fn add_context(input: I, ctx: &'static str, other: Self) -> Self {
        // Keep `Char`, `Tag` and `Context` errors unchanged and only replace less specific nom
        // errors with the context.
        if let ErrorKind::Nom(_) = &other.kind {
            InternalError::new(input, ErrorKind::Context(ctx))
        } else {
            other
        }
    }
}

impl<I, E> FromExternalError<I, E> for InternalError<I> {
    #[inline]
    fn from_external_error(input: I, kind: nom::error::ErrorKind, _e: E) -> Self {
        InternalError::new(input, ErrorKind::Nom(kind))
    }
}

impl<I> fmt::Display for InternalError<I>
where
    I: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at: {}", self.kind, self.input)
    }
}

impl<I> std::error::Error for InternalError<I> where I: fmt::Debug + fmt::Display {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
    /// The zero-based byte offset into the input where the error occured.
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    line: String,
    kind: ErrorKind,
    location: Location,
}

impl Error {
    pub(super) fn from_internal_error<'a>(input: &'a str, err: InternalError<&'a str>) -> Error {
        let substring = err.input;
        let offset = input.offset(substring);
        let prefix = &input.as_bytes()[..offset];

        // Find the line that includes the subslice:
        // Find the *last* newline before the substring starts
        let line_begin = prefix
            .iter()
            .rev()
            .position(|&b| b == b'\n')
            .map_or(0, |pos| offset - pos);

        // Find the full line after that newline
        let line = input[line_begin..]
            .lines()
            .next()
            .unwrap_or(&input[line_begin..])
            .trim_end();

        // Count the number of newlines in the first `offset` bytes of input
        let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

        // The (1-indexed) column number is the offset of our substring into that line
        let column_number = line.offset(substring) + 1;

        Error {
            line: line.to_owned(),
            kind: err.kind.into_owned(),
            location: Location {
                offset,
                line: line_number,
                col: column_number,
            },
        }
    }

    /// Returns the line from the input where the error occurs as a `&str`.
    ///
    /// Note that this returns the full line containing the invalid input. Use
    /// [`.location()`][Error::location] to obtain the column in which the error starts.
    pub fn line(&self) -> &str {
        &self.line
    }

    /// Returns the location in the input at which the error occured.
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// Returns the kind of the error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    fn spacing(&self) -> String {
        let line_str_len = format!("{}", self.location.line).len();
        " ".repeat(line_str_len)
    }

    fn message(&self) -> String {
        format!(
            "{} in line {}, col {}",
            self.kind, self.location.line, self.location.col
        )
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{s}--> {l}:{c}\n\
                 {s} |\n\
                 {l} | {line}\n\
                 {s} | {caret:>c$}---\n\
                 {s} |\n\
                 {s} = {message}",
            s = self.spacing(),
            l = self.location.line,
            c = self.location.col,
            line = self.line,
            caret = '^',
            message = self.message()
        )
    }
}
