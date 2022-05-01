use super::escape::{CharEscape, ESCAPE};
use std::io;
use unicode_xid::UnicodeXID;

/// This trait abstracts away serializing the HCL control characters, which allows the user to
/// optionally pretty print the HCL output.
pub trait Format {
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"null")
    }

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

    fn write_int<W, I>(&mut self, writer: &mut W, value: I) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        I: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        writer.write_all(s.as_bytes())
    }

    fn write_float<W, F>(&mut self, writer: &mut W, value: F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ryu::Float,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        writer.write_all(s.as_bytes())
    }

    fn write_quoted_string<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\"")?;
        self.write_escaped_string(writer, s)?;
        writer.write_all(b"\"")
    }

    fn write_string_fragment<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(s.as_bytes())
    }

    /// Ensures that `ident` is valid according to the [Unicode Standard Annex
    /// #31][unicode-standard] before writing it to the writer.
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

        if !first.is_xid_start() || !chars.all(UnicodeXID::is_xid_continue) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid identifier",
            ));
        }

        self.write_string_fragment(writer, ident)
    }

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

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_attribute_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b" = ")
    }

    fn end_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn begin_block_body<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;

    fn end_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write;
}

/// A pretty printing HCL formatter.
pub struct PrettyFormatter<'a> {
    first_element: bool,
    current_indent: usize,
    has_value: bool,
    indent: &'a [u8],
}

impl<'a> Default for PrettyFormatter<'a> {
    fn default() -> Self {
        PrettyFormatter::with_indent(b"  ")
    }
}

impl<'a> PrettyFormatter<'a> {
    /// Creates a `Formater` which will use the given indent for indenting nested HCL structures.
    pub fn with_indent(indent: &'a [u8]) -> Self {
        PrettyFormatter {
            first_element: false,
            current_indent: 0,
            has_value: false,
            indent,
        }
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

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"{")
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

    fn begin_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n")?;
        indent(writer, self.current_indent, self.indent)
    }

    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    fn begin_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        indent(writer, self.current_indent, self.indent)
    }

    fn end_attribute<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\n")
    }

    fn begin_block<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
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
