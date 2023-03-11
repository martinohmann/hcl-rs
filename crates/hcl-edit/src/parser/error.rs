use std::fmt;
use winnow::{
    error::{ContextError, FromExternalError, ParseError},
    stream::{AsBytes, Offset},
};

/// Represents a location in the parser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// The one-based line number of the error.
    pub line: usize,
    /// The one-based column number of the error.
    pub col: usize,
}

/// The result type used by this module.
pub type ParseResult<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Context {
    Expression(&'static str),
    Expected(Expected),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expected {
    Char(char),
    Literal(&'static str),
    Description(&'static str),
}

impl std::fmt::Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Char('\n') => write!(f, "newline"),
            Expected::Char(c) if c.is_ascii_control() => {
                write!(f, "`{}`", c.escape_debug())
            }
            Expected::Char(c) => write!(f, "`{c}`"),
            Expected::Literal(l) => write!(f, "`{l}`"),
            Expected::Description(d) => write!(f, "{d}"),
        }
    }
}

#[derive(Debug)]
pub struct InternalError<I> {
    input: I,
    context: Vec<Context>,
    cause: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<I> InternalError<I> {
    #[inline]
    pub(super) fn new(input: I) -> InternalError<I> {
        InternalError {
            input,
            context: Vec::new(),
            cause: None,
        }
    }
}

impl<I> PartialEq for InternalError<I>
where
    I: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && self.context == other.context
            && self.cause.as_ref().map(ToString::to_string)
                == other.cause.as_ref().map(ToString::to_string)
    }
}

impl<I> ParseError<I> for InternalError<I> {
    #[inline]
    fn from_error_kind(input: I, _kind: winnow::error::ErrorKind) -> Self {
        InternalError::new(input)
    }

    #[inline]
    fn from_char(input: I, _ch: char) -> Self {
        InternalError::new(input)
    }

    #[inline]
    fn append(self, _input: I, _kind: winnow::error::ErrorKind) -> Self {
        self
    }
}

impl<I> ContextError<I, Context> for InternalError<I> {
    #[inline]
    fn add_context(mut self, _input: I, ctx: Context) -> Self {
        self.context.push(ctx);
        self
    }
}

impl<I, E> FromExternalError<I, E> for InternalError<I>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[inline]
    fn from_external_error(input: I, _kind: winnow::error::ErrorKind, err: E) -> Self {
        InternalError {
            input,
            context: Vec::new(),
            cause: Some(Box::new(err)),
        }
    }
}

impl<I> fmt::Display for InternalError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let expression = self.context.iter().find_map(|c| match c {
            Context::Expression(c) => Some(c),
            _ => None,
        });

        let expected = self
            .context
            .iter()
            .filter_map(|c| match c {
                Context::Expected(c) => Some(c),
                _ => None,
            })
            .collect::<Vec<_>>();

        if let Some(expression) = expression {
            write!(f, "invalid {expression}; ")?;
        }

        if expected.is_empty() {
            f.write_str("unexpected token")?;
        } else {
            write!(f, "expected ")?;

            match expected.len() {
                0 => {}
                1 => write!(f, "{}", &expected[0])?,
                n => {
                    for (i, expected) in expected.iter().enumerate() {
                        if i == n - 1 {
                            f.write_str(" or ")?;
                        } else if i > 0 {
                            f.write_str(", ")?;
                        }

                        write!(f, "{expected}")?;
                    }
                }
            }
        }

        if let Some(cause) = &self.cause {
            write!(f, "; {cause}")?;
        }

        Ok(())
    }
}

impl<I> std::error::Error for InternalError<I> where I: fmt::Debug + fmt::Display {}

/// Error type returned when the parser encountered an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    pub(super) fn from_internal_error<I>(input: I, err: InternalError<I>) -> Error
    where
        I: AsBytes + Offset,
    {
        Error::new(ErrorInner::from_internal_error(input, err))
    }

    fn new(inner: ErrorInner) -> Error {
        Error {
            inner: Box::new(inner),
        }
    }

    /// Returns the line from the input where the error occurred.
    ///
    /// Note that this returns the full line containing the invalid input. Use
    /// [`.location()`][Error::location] to obtain the column in which the error starts.
    pub fn line(&self) -> &str {
        &self.inner.line
    }

    /// Returns the location in the input at which the error occurred.
    pub fn location(&self) -> &Location {
        &self.inner.location
    }

    /// Returns the zero-based byte offset into the input where the error occurred.
    pub fn offset(&self) -> usize {
        self.inner.offset
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErrorInner {
    message: String,
    line: String,
    location: Location,
    offset: usize,
}

impl ErrorInner {
    fn from_internal_error<I>(input: I, err: InternalError<I>) -> ErrorInner
    where
        I: AsBytes + Offset,
    {
        let offset = input.offset_to(&err.input);
        let (line, location) = locate_error(input.as_bytes(), offset);

        ErrorInner {
            message: err.to_string(),
            line: String::from_utf8_lossy(line).to_string(),
            offset,
            location,
        }
    }

    fn spacing(&self) -> String {
        " ".repeat(self.location.line.to_string().len())
    }
}

impl fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{s}--> HCL parse error in line {l}, column {c}\n\
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
            message = self.message,
        )
    }
}

fn locate_error(input: &[u8], offset: usize) -> (&[u8], Location) {
    let consumed_input = &input[..offset];
    let remaining_input = &input[offset..];

    // Find the line that includes the subslice:
    // Find the *last* newline before the remaining input starts
    let line_begin = consumed_input
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map_or(0, |pos| offset - pos);

    // Find the full line after that newline
    let line = input[line_begin..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(&input[line_begin..], |pos| {
            &input[line_begin..line_begin + pos]
        });

    // Count the number of newlines in the first `offset` bytes of input
    let line_number = consumed_input.iter().filter(|&&b| b == b'\n').count() + 1;

    // The (1-indexed) column number is the offset of the remaining input into that line
    let column_number = line.offset_to(remaining_input) + 1;

    (
        line,
        Location {
            line: line_number,
            col: column_number,
        },
    )
}
