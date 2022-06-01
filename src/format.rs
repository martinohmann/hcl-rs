//! Formatter implementation used by the [`Serializer`][Serializer] to construct HCL documents.
//!
//! [Serializer]: ser/struct.Serializer.html

use super::escape::{CharEscape, ESCAPE};
use std::io;
use unicode_ident::{is_xid_continue, is_xid_start};

/// This trait abstracts away serializing the HCL control characters, which allows the user to
/// optionally pretty print the HCL output.
pub trait Format {
    /// Writes `null` to the writer.
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"null")
    }

    /// Writes a boolean value to the writer.
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        writer.write_all(s)
    }

    /// Writes an integer value to the writer.
    fn write_int<W, I>(&mut self, writer: &mut W, value: I) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        I: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        writer.write_all(s.as_bytes())
    }

    /// Writes a float value to the writer.
    fn write_float<W, F>(&mut self, writer: &mut W, value: F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ryu::Float,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        writer.write_all(s.as_bytes())
    }

    /// Writes a quoted string to the writer. The quoted string will be escaped. See
    /// [`write_escaped_string`].
    fn write_quoted_string<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\"")?;
        self.write_escaped_string(writer, s)?;
        writer.write_all(b"\"")
    }

    /// Writes a string fragment to the writer. No escaping occurs.
    fn write_string_fragment<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(s.as_bytes())
    }

    /// Writes an identifier to the writer. Ensures that `ident` is valid according to the [Unicode
    /// Standard Annex #31][unicode-standard] before writing it to the writer.
    ///
    /// [unicode-standard]: http://www.unicode.org/reports/tr31/
    fn write_ident<W>(&mut self, writer: &mut W, ident: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
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

        self.write_string_fragment(writer, ident)
    }

    /// Writes a string to the writer and escapes control characters and quotes that might be
    /// contained in it.
    fn write_escaped_string<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let bytes = value.as_bytes();

        let mut start = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }

            if start < i {
                self.write_string_fragment(writer, &value[start..i])?;
            }

            let char_escape = CharEscape::from_escape_table(escape, byte);
            char_escape.write_escaped(writer)?;

            start = i + 1;
        }

        if start != bytes.len() {
            self.write_string_fragment(writer, &value[start..])?;
        }

        Ok(())
    }

    /// Starts an interpolated string.
    fn begin_interpolated_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\"${")
    }

    /// Ends an interpolated string.
    fn end_interpolated_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"}\"")
    }

    /// Signals the start of an array to the formatter.
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an array value to the formatter.
    fn begin_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the end of an array value to the formatter.
    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the end of an array to the formatter.
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an object to the formatter.
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an object key to the formatter.
    fn begin_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an object value to the formatter.
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    /// Signals the end of an object value to the formatter.
    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.end_array_value(writer)
    }

    /// Signals the end of an object to the formatter.
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an attribute to the formatter.
    fn begin_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of an attribute value to the formatter.
    fn begin_attribute_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    /// Signals the end of an attribute to the formatter.
    fn end_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of a block to the formatter.
    fn begin_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the start of a block body to the formatter.
    fn begin_block_body<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    /// Signals the end of a block to the formatter.
    fn end_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;
}

#[derive(PartialEq)]
enum FormatState {
    Initial,
    AttributeEnd,
    BlockEnd,
}

/// A pretty printing HCL formatter.
pub struct PrettyFormatter<'a> {
    state: FormatState,
    first_element: bool,
    current_indent: usize,
    has_value: bool,
    indent: &'a [u8],
    dense: bool,
}

impl<'a> Default for PrettyFormatter<'a> {
    fn default() -> Self {
        PrettyFormatter::builder().build()
    }
}

/// A builder to create a `PrettyFormatter`.
pub struct PrettyFormatterBuilder<'a> {
    indent: &'a [u8],
    dense: bool,
}

impl<'a> PrettyFormatterBuilder<'a> {
    /// Creates a new [`PrettyFormatterBuilder`] to start building a new `PrettyFormatter`.
    pub fn new() -> Self {
        PrettyFormatterBuilder {
            indent: b"  ",
            dense: false,
        }
    }

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

    /// Consumes the `PrettyFormatterBuilder` and turns it into a `PrettyFormatter`.
    pub fn build(self) -> PrettyFormatter<'a> {
        PrettyFormatter {
            state: FormatState::Initial,
            first_element: false,
            current_indent: 0,
            has_value: false,
            indent: self.indent,
            dense: self.dense,
        }
    }
}

impl<'a> PrettyFormatter<'a> {
    /// Creates a new [`PrettyFormatterBuilder`] to start building a new `PrettyFormatter`.
    pub fn builder() -> PrettyFormatterBuilder<'a> {
        PrettyFormatterBuilder::new()
    }
}

impl<'a> Format for PrettyFormatter<'a> {
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        self.first_element = true;
        writer.write_all(b"[")
    }

    fn begin_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if self.first_element {
            self.first_element = false;
            writer.write_all(b"\n")?;
        } else {
            writer.write_all(b",\n")?;
        }

        indent(writer, self.current_indent, self.indent)
    }

    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;

        if self.has_value {
            writer.write_all(b"\n")?;
            indent(writer, self.current_indent, self.indent)?;
        }

        writer.write_all(b"]")
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"{")
    }

    fn begin_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n")?;
        indent(writer, self.current_indent, self.indent)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;

        if self.has_value {
            writer.write_all(b"\n")?;
            indent(writer, self.current_indent, self.indent)?;
        }

        writer.write_all(b"}")
    }

    fn begin_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !self.dense && self.state == FormatState::BlockEnd {
            writer.write_all(b"\n")?;
        }

        indent(writer, self.current_indent, self.indent)
    }

    fn end_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.state = FormatState::AttributeEnd;
        writer.write_all(b"\n")
    }

    fn begin_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !self.dense
            && matches!(
                self.state,
                FormatState::AttributeEnd | FormatState::BlockEnd
            )
        {
            writer.write_all(b"\n")?;
        }

        indent(writer, self.current_indent, self.indent)
    }

    fn begin_block_body<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        writer.write_all(b" {\n")
    }

    fn end_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.state = FormatState::BlockEnd;
        self.current_indent -= 1;
        indent(writer, self.current_indent, self.indent)?;
        writer.write_all(b"}\n")
    }
}

fn indent<W>(writer: &mut W, n: usize, s: &[u8]) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    for _ in 0..n {
        writer.write_all(s)?;
    }

    Ok(())
}
