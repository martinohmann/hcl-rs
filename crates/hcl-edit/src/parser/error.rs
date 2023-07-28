use super::prelude::*;

use std::fmt::{self, Write};
use winnow::error::ParseError;
use winnow::stream::Offset;

/// Error type returned when the parser encountered an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl Error {
    pub(super) fn from_parse_error(err: &ParseError<Input, ContextError>) -> Error {
        Error::new(ErrorInner::from_parse_error(err))
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
    fn from_parse_error(err: &ParseError<Input, ContextError>) -> ErrorInner {
        let (line, location) = locate_error(err);

        ErrorInner {
            message: format_context_error(err.inner()),
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

fn locate_error<'a>(err: &ParseError<Input<'a>, ContextError>) -> (&'a [u8], Location) {
    let offset = err.offset();
    let input = err.input();
    let remaining_input = &input[offset..];
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
    let column = remaining_input.offset_from(&line_context) + 1;

    (
        line_context,
        Location {
            line,
            column,
            offset,
        },
    )
}

// This is almost identical to `ContextError::to_string` but produces a slightly different format
// which does not contain line breaks and emits "unexpected token" when there was no expectation in
// the context.
fn format_context_error(err: &ContextError) -> String {
    let mut buf = String::new();

    let label = err.context().find_map(|c| match c {
        StrContext::Label(c) => Some(c),
        _ => None,
    });

    let expected = err
        .context()
        .filter_map(|c| match c {
            StrContext::Expected(c) => Some(c),
            _ => None,
        })
        .collect::<Vec<_>>();

    if let Some(label) = label {
        _ = write!(buf, "invalid {label}; ");
    }

    if expected.is_empty() {
        _ = buf.write_str("unexpected token");
    } else {
        _ = write!(buf, "expected ");

        match expected.len() {
            0 => {}
            1 => {
                _ = write!(buf, "{}", &expected[0]);
            }
            n => {
                for (i, expected) in expected.iter().enumerate() {
                    if i == n - 1 {
                        _ = buf.write_str(" or ");
                    } else if i > 0 {
                        _ = buf.write_str(", ");
                    }

                    _ = write!(buf, "{expected}");
                }
            }
        };
    }

    if let Some(cause) = err.cause() {
        _ = write!(buf, "; {cause}");
    }

    buf
}
