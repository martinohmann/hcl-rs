//! Contains the logic to format HCL data structure.

// @NOTE(mohmann): This module is not exported yet since it is subject to change due to a bigger
// multi-step refactoring. It will eventually replace the formatting code inside serializer
// implementations.

use crate::{
    ser,
    structure::{
        Attribute, Block, BlockLabel, Body, Expression, Identifier, ObjectKey, RawExpression,
        Structure,
    },
    Map, Number, Result, Value,
};
use std::io;

/// A trait to format data structures as HCL.
pub trait Format {
    /// Formats a HCL structure using a formatter and writes the result to the provided writer.
    ///
    /// ## Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `io::Error`.
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format;
}

impl Format for Body {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        for structure in self.iter() {
            structure.format(writer, fmt)?;
        }

        Ok(())
    }
}

impl Format for Structure {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            Structure::Attribute(attr) => attr.format(writer, fmt),
            Structure::Block(block) => block.format(writer, fmt),
        }
    }
}

impl Format for Attribute {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.begin_attribute(writer)?;
        fmt.write_ident(writer, &self.key)?;
        fmt.begin_attribute_value(writer)?;
        self.expr.format(writer, fmt)?;
        fmt.end_attribute(writer)
    }
}

impl Format for Block {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.begin_block(writer)?;
        fmt.write_ident(writer, &self.identifier)?;

        for label in &self.labels {
            writer.write_all(b" ")?;
            label.format(writer, fmt)?;
        }

        fmt.begin_block_body(writer)?;
        self.body.format(writer, fmt)?;
        fmt.end_block(writer)
    }
}

impl Format for BlockLabel {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            BlockLabel::Identifier(ident) => ident.format(writer, fmt),
            BlockLabel::String(string) => string.format(writer, fmt),
        }
    }
}

impl Format for Expression {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            Expression::Null => fmt.write_null(writer),
            Expression::Bool(b) => fmt.write_bool(writer, *b),
            Expression::Number(num) => num.format(writer, fmt),
            Expression::String(string) => string.format(writer, fmt),
            Expression::Array(array) => format_array(writer, fmt, array),
            Expression::Object(object) => format_object(writer, fmt, object),
            Expression::Raw(raw) => raw.format(writer, fmt),
        }
    }
}

impl Format for Value {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            Value::Null => fmt.write_null(writer),
            Value::Bool(b) => fmt.write_bool(writer, *b),
            Value::Number(num) => num.format(writer, fmt),
            Value::String(string) => string.format(writer, fmt),
            Value::Array(array) => format_array(writer, fmt, array),
            Value::Object(object) => format_object(writer, fmt, object),
        }
    }
}

impl Format for Number {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match *self {
            Number::PosInt(pos) => fmt.write_int(writer, pos),
            Number::NegInt(neg) => fmt.write_int(writer, neg),
            Number::Float(float) => fmt.write_float(writer, float),
        }
    }
}

impl Format for ObjectKey {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            ObjectKey::Identifier(ident) => ident.format(writer, fmt),
            ObjectKey::String(string) => string.format(writer, fmt),
            ObjectKey::RawExpression(raw) => {
                fmt.begin_interpolated_string(writer)?;
                raw.format(writer, fmt)?;
                fmt.end_interpolated_string(writer)
            }
        }
    }
}

impl Format for RawExpression {
    fn format<W, F>(&self, writer: &mut W, _: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        writer.write_all(self.as_str().as_bytes())
    }
}

impl Format for Identifier {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.write_ident(writer, self.as_str())
    }
}

impl Format for String {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> io::Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.write_quoted_string(writer, self)
    }
}

fn format_array<W, F, T>(writer: &mut W, fmt: &mut F, array: &[T]) -> io::Result<()>
where
    W: ?Sized + io::Write,
    F: ?Sized + ser::Format,
    T: Format,
{
    fmt.begin_array(writer)?;

    for value in array {
        fmt.begin_array_value(writer)?;
        value.format(writer, fmt)?;
        fmt.end_array_value(writer)?;
    }

    fmt.end_array(writer)
}

fn format_object<W, F, K, V>(writer: &mut W, fmt: &mut F, object: &Map<K, V>) -> io::Result<()>
where
    W: ?Sized + io::Write,
    F: ?Sized + ser::Format,
    K: Format,
    V: Format,
{
    fmt.begin_object(writer)?;

    for (key, value) in object {
        fmt.begin_object_key(writer)?;
        key.format(writer, fmt)?;
        fmt.begin_object_value(writer)?;
        value.format(writer, fmt)?;
        fmt.end_object_value(writer)?;
    }

    fmt.end_object(writer)
}

/// Format the given value as an HCL byte vector.
///
/// # Errors
///
/// Formatting fails if the data structure contains malformed data in certain fields.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Format,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serialize the given value as an HCL string.
///
/// # Errors
///
/// Formatting fails if the data structure contains malformed data in certain fields.
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

/// Format the given value as HCL and write it into the IO stream.
///
/// # Errors
///
/// Formatting fails if the data structure contains malformed data in certain fields or if any
/// operation on the writer fails.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Format,
{
    let mut fmt = ser::PrettyFormatter::default();
    value.format(&mut writer, &mut fmt)?;
    Ok(())
}
