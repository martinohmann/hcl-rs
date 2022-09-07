use super::{private, Format, Formatter};
use crate::{
    structure::{
        Attribute, Block, BlockLabel, Body, Expression, Heredoc, HeredocStripMode, Identifier,
        ObjectKey, RawExpression, Structure, TemplateExpr,
    },
    Map, Number, Result, Value,
};
use std::io::{self, Write};

impl private::Sealed for Body {}

impl Format for Body {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        for structure in self.iter() {
            structure.format(fmt)?;
        }

        Ok(())
    }
}

impl private::Sealed for Structure {}

impl Format for Structure {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Structure::Attribute(attr) => attr.format(fmt),
            Structure::Block(block) => block.format(fmt),
        }
    }
}

impl private::Sealed for Attribute {}

impl Format for Attribute {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.begin_attribute()?;
        fmt.write_ident(&self.key)?;
        fmt.begin_attribute_value()?;
        self.expr.format(fmt)?;
        fmt.end_attribute()?;
        Ok(())
    }
}

impl private::Sealed for Block {}

impl Format for Block {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.begin_block()?;
        fmt.write_ident(&self.identifier)?;

        for label in &self.labels {
            fmt.write_all(b" ")?;
            label.format(fmt)?;
        }

        fmt.begin_block_body()?;
        self.body.format(fmt)?;
        fmt.end_block()?;
        Ok(())
    }
}

impl private::Sealed for BlockLabel {}

impl Format for BlockLabel {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            BlockLabel::Identifier(ident) => ident.format(fmt),
            BlockLabel::String(string) => string.format(fmt),
        }
    }
}

impl private::Sealed for Expression {}

impl Format for Expression {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Expression::Null => Ok(fmt.write_null()?),
            Expression::Bool(b) => Ok(fmt.write_bool(*b)?),
            Expression::Number(num) => num.format(fmt),
            Expression::String(string) => string.format(fmt),
            Expression::Array(array) => format_array(fmt, array),
            Expression::Object(object) => format_object(fmt, object),
            Expression::Raw(raw) => raw.format(fmt),
            Expression::TemplateExpr(expr) => expr.format(fmt),
            Expression::VariableExpr(ident) => ident.format(fmt),
        }
    }
}

impl private::Sealed for Value {}

impl Format for Value {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Value::Null => Ok(fmt.write_null()?),
            Value::Bool(b) => Ok(fmt.write_bool(*b)?),
            Value::Number(num) => num.format(fmt),
            Value::String(string) => string.format(fmt),
            Value::Array(array) => format_array(fmt, array),
            Value::Object(object) => format_object(fmt, object),
        }
    }
}

impl private::Sealed for Number {}

impl Format for Number {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match *self {
            Number::PosInt(pos) => fmt.write_int(pos)?,
            Number::NegInt(neg) => fmt.write_int(neg)?,
            Number::Float(float) => fmt.write_float(float)?,
        }

        Ok(())
    }
}

impl private::Sealed for ObjectKey {}

impl Format for ObjectKey {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            ObjectKey::Identifier(ident) => ident.format(fmt),
            ObjectKey::String(string) => string.format(fmt),
            ObjectKey::RawExpression(raw) => {
                fmt.begin_interpolated_string()?;
                raw.format(fmt)?;
                fmt.end_interpolated_string()?;
                Ok(())
            }
        }
    }
}

impl private::Sealed for RawExpression {}

impl Format for RawExpression {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_all(self.as_str().as_bytes())?;
        Ok(())
    }
}

impl private::Sealed for TemplateExpr {}

impl Format for TemplateExpr {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            TemplateExpr::QuotedString(string) => string.format(fmt),
            TemplateExpr::Heredoc(heredoc) => heredoc.format(fmt),
        }
    }
}

impl private::Sealed for Heredoc {}

impl Format for Heredoc {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        let delimiter = self.delimiter.as_str();

        fmt.write_string_fragment(self.strip.as_str())?;
        fmt.write_string_fragment(delimiter)?;
        fmt.write_all(b"\n")?;
        fmt.write_string_fragment(&self.template)?;

        if !self.template.ends_with('\n') {
            fmt.write_all(b"\n")?;
        }

        match self.strip {
            HeredocStripMode::None => {
                fmt.write_string_fragment(delimiter)?;
            }
            HeredocStripMode::Indent => {
                fmt.write_indented(fmt.current_indent, delimiter)?;
            }
        }

        Ok(())
    }
}

impl private::Sealed for Identifier {}

impl Format for Identifier {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_ident(self.as_str())?;
        Ok(())
    }
}

impl private::Sealed for String {}

impl Format for String {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_quoted_string(self)?;
        Ok(())
    }
}

fn format_array<W, T>(fmt: &mut Formatter<W>, array: &[T]) -> Result<()>
where
    W: io::Write,
    T: Format,
{
    fmt.begin_array()?;

    for value in array {
        fmt.begin_array_value()?;
        value.format(fmt)?;
        fmt.end_array_value()?;
    }

    fmt.end_array()?;
    Ok(())
}

fn format_object<W, K, V>(fmt: &mut Formatter<W>, object: &Map<K, V>) -> Result<()>
where
    W: io::Write,
    K: Format,
    V: Format,
{
    fmt.begin_object()?;

    for (key, value) in object {
        fmt.begin_object_key()?;
        key.format(fmt)?;
        fmt.begin_object_value()?;
        value.format(fmt)?;
        fmt.end_object_value()?;
    }

    fmt.end_object()?;
    Ok(())
}
