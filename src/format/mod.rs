//! Contains the logic to format HCL data structures.

mod escape;
mod impls;

use self::escape::{CharEscape, ESCAPE};
use crate::Result;
use std::io::{self, Write};
use std::marker::PhantomData;
use unicode_ident::{is_xid_continue, is_xid_start};

mod private {
    pub trait Sealed {}
}

/// A trait to format data structures as HCL.
///
/// This trait is sealed to prevent implementation outside of this crate.
pub trait Format: private::Sealed {
    /// Formats a HCL structure using a formatter and writes the result to the provided writer.
    ///
    /// ## Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `Error`.
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write;
}

#[derive(PartialEq)]
enum FormatState {
    Initial,
    AttributeStart,
    AttributeEnd,
    BlockStart,
    BlockEnd,
    BlockBodyStart,
}

/// A pretty printing HCL formatter.
///
/// ## Example
///
/// The `.builder()` method can be used to construct a `Formatter` for use with a
/// [`Serializer`][Serializer]:
///
/// ```
/// use hcl::{format::Formatter, ser::Serializer};
/// # let mut writer = Vec::new();
///
/// let formatter = Formatter::builder()
///     .indent(b"  ")
///     .dense(false)
///     .build(&mut writer);
///
/// let ser = Serializer::with_formatter(formatter);
/// ```
///
/// [Serializer]: ../ser/struct.Serializer.html
pub struct Formatter<'a, W> {
    writer: W,
    state: FormatState,
    first_element: bool,
    current_indent: usize,
    has_value: bool,
    indent: &'a [u8],
    dense: bool,
}

/// A builder to create a `Formatter`.
///
/// See the documentation of [`Formatter`] for a usage example.
pub struct FormatterBuilder<'a, W> {
    indent: &'a [u8],
    dense: bool,
    marker: PhantomData<W>,
}

impl<'a, W> FormatterBuilder<'a, W> {
    /// Set the indent for indenting nested HCL structures.
    pub fn indent(mut self, indent: &'a [u8]) -> Self {
        self.indent = indent;
        self
    }

    /// If set, blocks are not visually separated by empty lines from attributes and adjacent
    /// blocks.
    pub fn dense(mut self, yes: bool) -> Self {
        self.dense = yes;
        self
    }

    /// Consumes the `FormatterBuilder` and turns it into a `Formatter` which writes HCL to the
    /// provided writer.
    pub fn build(self, writer: W) -> Formatter<'a, W>
    where
        W: io::Write,
    {
        Formatter {
            writer,
            state: FormatState::Initial,
            first_element: false,
            current_indent: 0,
            has_value: false,
            indent: self.indent,
            dense: self.dense,
        }
    }
}

// Public API.
impl<'a, W> Formatter<'a, W>
where
    W: io::Write,
{
    /// Creates a new `Formatter` which writes HCL to the provided writer.
    pub fn new(writer: W) -> Formatter<'a, W> {
        Formatter::builder().build(writer)
    }
}

// Public API.
impl<'a, W> Formatter<'a, W> {
    /// Creates a new [`FormatterBuilder`] to start building a new `Formatter`.
    pub fn builder() -> FormatterBuilder<'a, W> {
        FormatterBuilder {
            indent: b"  ",
            dense: false,
            marker: PhantomData,
        }
    }

    /// Consumes `self` and returns the wrapped writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

// Internal formatter API.
impl<'a, W> Formatter<'a, W>
where
    W: io::Write,
{
    /// Writes `null` to the writer.
    fn write_null(&mut self) -> io::Result<()> {
        self.write_all(b"null")
    }

    /// Writes a boolean value to the writer.
    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        self.write_all(s)
    }

    /// Writes an integer value to the writer.
    fn write_int<T>(&mut self, value: T) -> io::Result<()>
    where
        T: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write_all(s.as_bytes())
    }

    /// Writes a float value to the writer.
    fn write_float<T>(&mut self, value: T) -> io::Result<()>
    where
        T: ryu::Float,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        self.write_all(s.as_bytes())
    }

    /// Writes a quoted string to the writer. The quoted string will be escaped.
    fn write_quoted_string(&mut self, s: &str) -> io::Result<()> {
        self.write_all(b"\"")?;
        self.write_escaped_string(s)?;
        self.write_all(b"\"")
    }

    /// Writes a string fragment to the writer. No escaping occurs.
    fn write_string_fragment(&mut self, s: &str) -> io::Result<()> {
        self.write_all(s.as_bytes())
    }

    /// Writes an identifier to the writer. Ensures that `ident` is valid according to the [Unicode
    /// Standard Annex #31][unicode-standard] before writing it to the writer.
    ///
    /// [unicode-standard]: http://www.unicode.org/reports/tr31/
    fn write_ident(&mut self, ident: &str) -> io::Result<()> {
        if ident.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "identifiers must not be empty",
            ));
        }

        let mut chars = ident.chars();
        let first = chars.next().unwrap();

        if !is_xid_start(first) || !chars.all(is_xid_continue) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid identifier",
            ));
        }

        self.write_string_fragment(ident)
    }

    /// Writes a string to the writer and escapes control characters and quotes that might be
    /// contained in it.
    fn write_escaped_string(&mut self, value: &str) -> io::Result<()> {
        let bytes = value.as_bytes();

        let mut start = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }

            if start < i {
                self.write_string_fragment(&value[start..i])?;
            }

            let char_escape = CharEscape::from_escape_table(escape, byte);
            char_escape.write_escaped(&mut self.writer)?;

            start = i + 1;
        }

        if start != bytes.len() {
            self.write_string_fragment(&value[start..])?;
        }

        Ok(())
    }

    /// Starts an interpolated string.
    fn begin_interpolated_string(&mut self) -> io::Result<()> {
        self.write_all(b"\"${")
    }

    /// Ends an interpolated string.
    fn end_interpolated_string(&mut self) -> io::Result<()> {
        self.write_all(b"}\"")
    }

    /// Signals the start of an array to the formatter.
    fn begin_array(&mut self) -> io::Result<()> {
        self.current_indent += 1;
        self.has_value = false;
        self.first_element = true;
        self.write_all(b"[")
    }

    /// Signals the start of an array value to the formatter.
    fn begin_array_value(&mut self) -> io::Result<()> {
        if self.first_element {
            self.first_element = false;
            self.write_all(b"\n")?;
        } else {
            self.write_all(b",\n")?;
        }

        self.write_indent(self.current_indent, self.indent)
    }

    /// Signals the end of an array value to the formatter.
    fn end_array_value(&mut self) -> io::Result<()> {
        self.has_value = true;
        Ok(())
    }

    /// Signals the end of an array to the formatter.
    fn end_array(&mut self) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            self.write_all(b"\n")?;
            self.write_indent(self.current_indent, self.indent)?;
        }

        self.write_all(b"]")
    }

    /// Signals the start of an object to the formatter.
    fn begin_object(&mut self) -> io::Result<()> {
        self.current_indent += 1;
        self.has_value = false;
        self.write_all(b"{")
    }

    /// Signals the start of an object key to the formatter.
    fn begin_object_key(&mut self) -> io::Result<()> {
        self.write_all(b"\n")?;
        self.write_indent(self.current_indent, self.indent)
    }

    /// Signals the start of an object value to the formatter.
    fn begin_object_value(&mut self) -> io::Result<()> {
        self.write_all(b" = ")
    }

    /// Signals the end of an object value to the formatter.
    fn end_object_value(&mut self) -> io::Result<()> {
        self.end_array_value()
    }

    /// Signals the end of an object to the formatter.
    fn end_object(&mut self) -> io::Result<()> {
        self.current_indent -= 1;

        if self.has_value {
            self.write_all(b"\n")?;
            self.write_indent(self.current_indent, self.indent)?;
        }

        self.write_all(b"}")
    }

    /// Signals the start of an attribute to the formatter.
    fn begin_attribute(&mut self) -> io::Result<()> {
        self.maybe_write_newline(FormatState::AttributeStart)?;
        self.write_indent(self.current_indent, self.indent)
    }

    /// Signals the start of an attribute value to the formatter.
    fn begin_attribute_value(&mut self) -> io::Result<()> {
        self.write_all(b" = ")
    }

    /// Signals the end of an attribute to the formatter.
    fn end_attribute(&mut self) -> io::Result<()> {
        self.state = FormatState::AttributeEnd;
        self.write_all(b"\n")
    }

    /// Signals the start of a block to the formatter.
    fn begin_block(&mut self) -> io::Result<()> {
        self.maybe_write_newline(FormatState::BlockStart)?;
        self.write_indent(self.current_indent, self.indent)
    }

    /// Signals the start of a block body to the formatter.
    fn begin_block_body(&mut self) -> io::Result<()> {
        self.current_indent += 1;
        self.state = FormatState::BlockBodyStart;
        self.write_all(b" {")
    }

    /// Signals the end of a block to the formatter.
    fn end_block(&mut self) -> io::Result<()> {
        self.state = FormatState::BlockEnd;
        self.current_indent -= 1;
        self.write_indent(self.current_indent, self.indent)?;
        self.write_all(b"}\n")
    }

    // Conditionally writes a newline character depending on the formatter configuration and the
    // current and next state. Updates the state to `next_state`.
    fn maybe_write_newline(&mut self, next_state: FormatState) -> io::Result<()> {
        let newline = match &self.state {
            FormatState::AttributeEnd if !self.dense => {
                matches!(next_state, FormatState::BlockStart)
            }
            FormatState::BlockEnd if !self.dense => {
                matches!(
                    next_state,
                    FormatState::BlockStart | FormatState::AttributeStart
                )
            }
            other => matches!(other, FormatState::BlockBodyStart),
        };

        if newline {
            self.write_all(b"\n")?;
        }

        self.state = next_state;
        Ok(())
    }

    fn write_indent(&mut self, n: usize, s: &[u8]) -> io::Result<()> {
        for _ in 0..n {
            self.write_all(s)?;
        }

        Ok(())
    }
}

impl<'a, W> io::Write for Formatter<'a, W>
where
    W: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Format the given value as an HCL byte vector.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_vec`](crate::to_vec) instead.
///
/// # Errors
///
/// Formatting fails if the value contains strings that cannot be used as valid HCL identifiers.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Format,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Format the given value as an HCL string.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_string`](crate::to_string) instead.
///
/// # Errors
///
/// Formatting fails if the value contains strings that cannot be used as valid HCL identifiers.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Format,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Format the given value as HCL into the IO stream.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_writer`](crate::to_writer) instead.
///
/// # Errors
///
/// Formatting fails if any operation on the writer fails or if the value contains strings that
/// cannot be used as valid HCL identifiers.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Format,
{
    let mut formatter = Formatter::new(writer);
    value.format(&mut formatter)
}
