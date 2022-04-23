use std::io;

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

    fn write_str<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"\"")?;
        // @TODO(mohmann): handle escaping
        writer.write_all(s.as_bytes())?;
        writer.write_all(b"\"")
    }

    fn write_ident<W>(&mut self, writer: &mut W, ident: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        // @TODO(mohmann): validate chars
        writer.write_all(ident.as_bytes())
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
}

/// A compact HCL formatter.
#[derive(Default)]
pub struct CompactFormatter {
    first_element: bool,
    has_value: bool,
}

impl Format for CompactFormatter {
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.first_element = true;
        writer.write_all(b"[")
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b"]")
    }

    fn begin_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if self.first_element {
            self.first_element = false;
        } else {
            writer.write_all(b", ")?;
        }

        Ok(())
    }

    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = false;
        self.first_element = true;
        writer.write_all(b"{")
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if self.has_value {
            writer.write_all(b" ")?;
        }

        writer.write_all(b"}")
    }

    fn begin_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if self.first_element {
            self.first_element = false;
            writer.write_all(b" ")
        } else {
            writer.write_all(b", ")
        }
    }

    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
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
