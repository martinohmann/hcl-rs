use super::{private, Format};
use crate::{
    ser,
    structure::{
        Attribute, Block, BlockLabel, Body, Expression, Identifier, ObjectKey, RawExpression,
        Structure,
    },
    Map, Number, Result, Value,
};
use std::io;

impl private::Sealed for Body {}

impl Format for Body {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
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

impl private::Sealed for Structure {}

impl Format for Structure {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
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

impl private::Sealed for Attribute {}

impl Format for Attribute {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.begin_attribute(writer)?;
        fmt.write_ident(writer, &self.key)?;
        fmt.begin_attribute_value(writer)?;
        self.expr.format(writer, fmt)?;
        fmt.end_attribute(writer)?;
        Ok(())
    }
}

impl private::Sealed for Block {}

impl Format for Block {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
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
        fmt.end_block(writer)?;
        Ok(())
    }
}

impl private::Sealed for BlockLabel {}

impl Format for BlockLabel {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
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

impl private::Sealed for Expression {}

impl Format for Expression {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            Expression::Null => Ok(fmt.write_null(writer)?),
            Expression::Bool(b) => Ok(fmt.write_bool(writer, *b)?),
            Expression::Number(num) => num.format(writer, fmt),
            Expression::String(string) => string.format(writer, fmt),
            Expression::Array(array) => format_array(writer, fmt, array),
            Expression::Object(object) => format_object(writer, fmt, object),
            Expression::Raw(raw) => raw.format(writer, fmt),
        }
    }
}

impl private::Sealed for Value {}

impl Format for Value {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match self {
            Value::Null => Ok(fmt.write_null(writer)?),
            Value::Bool(b) => Ok(fmt.write_bool(writer, *b)?),
            Value::Number(num) => num.format(writer, fmt),
            Value::String(string) => string.format(writer, fmt),
            Value::Array(array) => format_array(writer, fmt, array),
            Value::Object(object) => format_object(writer, fmt, object),
        }
    }
}

impl private::Sealed for Number {}

impl Format for Number {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        match *self {
            Number::PosInt(pos) => fmt.write_int(writer, pos)?,
            Number::NegInt(neg) => fmt.write_int(writer, neg)?,
            Number::Float(float) => fmt.write_float(writer, float)?,
        }

        Ok(())
    }
}

impl private::Sealed for ObjectKey {}

impl Format for ObjectKey {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
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
                fmt.end_interpolated_string(writer)?;
                Ok(())
            }
        }
    }
}

impl private::Sealed for RawExpression {}

impl Format for RawExpression {
    fn format<W, F>(&self, writer: &mut W, _: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        writer.write_all(self.as_str().as_bytes())?;
        Ok(())
    }
}

impl private::Sealed for Identifier {}

impl Format for Identifier {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.write_ident(writer, self.as_str())?;
        Ok(())
    }
}

impl private::Sealed for String {}

impl Format for String {
    fn format<W, F>(&self, writer: &mut W, fmt: &mut F) -> Result<()>
    where
        W: ?Sized + io::Write,
        F: ?Sized + ser::Format,
    {
        fmt.write_quoted_string(writer, self)?;
        Ok(())
    }
}

fn format_array<W, F, T>(writer: &mut W, fmt: &mut F, array: &[T]) -> Result<()>
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

    fmt.end_array(writer)?;
    Ok(())
}

fn format_object<W, F, K, V>(writer: &mut W, fmt: &mut F, object: &Map<K, V>) -> Result<()>
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

    fmt.end_object(writer)?;
    Ok(())
}
