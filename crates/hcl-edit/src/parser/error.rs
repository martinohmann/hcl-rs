use super::context::Context;
use std::fmt;
use winnow::{
    error::{ContextError, FromExternalError},
    stream::{AsBytes, Offset},
};

/// Error type returned when the parser encountered an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    pub(super) fn from_parse_error<I>(input: I, err: ParseError<I>) -> Error
    where
        I: AsBytes + Offset,
    {
        Error::new(ErrorInner::from_parse_error(input, err))
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
}

impl ErrorInner {
    fn from_parse_error<I>(input: I, err: ParseError<I>) -> ErrorInner
    where
        I: AsBytes + Offset,
    {
        let (line, location) = locate_error(input.as_bytes(), err.input.as_bytes());

        ErrorInner {
            message: err.to_string(),
            line: String::from_utf8_lossy(line).to_string(),
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
            c = self.location.column,
            line = self.line,
            caret = '^',
            message = self.message,
        )
    }
}

/// Represents a location in the parser input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    line: usize,
    column: usize,
    offset: usize,
}

impl Location {
    /// Returns the line number (one-based) in the parser input.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number (one-based) in the parser input.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns the byte offset (zero-based) in the parser input.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

fn locate_error<'a>(input: &'a [u8], remaining_input: &'a [u8]) -> (&'a [u8], Location) {
    let offset = input.offset_to(remaining_input);
    let consumed_input = &input[..offset];

    // Find the line that includes the subslice:
    // Find the *last* newline before the remaining input starts
    let line_begin = consumed_input
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map_or(0, |pos| offset - pos);

    // Find the full line after that newline
    let line_context = input[line_begin..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(&input[line_begin..], |pos| {
            &input[line_begin..line_begin + pos]
        });

    // Count the number of newlines in the first `offset` bytes of input
    let line = consumed_input.iter().filter(|&&b| b == b'\n').count() + 1;

    // The (1-indexed) column number is the offset of the remaining input into that line.
    let column = line_context.offset_to(remaining_input) + 1;

    (
        line_context,
        Location {
            line,
            column,
            offset,
        },
    )
}

#[derive(Debug)]
pub(super) struct ParseError<I> {
    input: I,
    context: Vec<Context>,
    cause: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl<I> ParseError<I> {
    #[inline]
    pub(super) fn new(input: I) -> ParseError<I> {
        ParseError {
            input,
            context: Vec::new(),
            cause: None,
        }
    }
}

impl<I> PartialEq for ParseError<I>
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

impl<I> winnow::error::ParseError<I> for ParseError<I> {
    #[inline]
    fn from_error_kind(input: I, _kind: winnow::error::ErrorKind) -> Self {
        ParseError::new(input)
    }

    #[inline]
    fn append(self, _input: I, _kind: winnow::error::ErrorKind) -> Self {
        self
    }
}

impl<I> ContextError<I, Context> for ParseError<I> {
    #[inline]
    fn add_context(mut self, _input: I, ctx: Context) -> Self {
        self.context.push(ctx);
        self
    }
}

impl<I, E> FromExternalError<I, E> for ParseError<I>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[inline]
    fn from_external_error(input: I, _kind: winnow::error::ErrorKind, err: E) -> Self {
        ParseError {
            input,
            context: Vec::new(),
            cause: Some(Box::new(err)),
        }
    }
}

impl<I> fmt::Display for ParseError<I> {
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

impl<I> std::error::Error for ParseError<I> where I: fmt::Debug + fmt::Display {}
