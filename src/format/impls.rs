use super::{private, Format, Formatter};
use crate::{structure::*, Number, Result, Value};
use std::io::{self, Write};

impl<T> private::Sealed for &T where T: Format {}

impl<T> Format for &T
where
    T: Format,
{
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        (*self).format(fmt)
    }
}

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
            Expression::Array(array) => format_array(fmt, array.iter()),
            Expression::Object(object) => format_object(fmt, object.iter()),
            Expression::Raw(raw) => raw.format(fmt),
            Expression::TemplateExpr(expr) => expr.format(fmt),
            Expression::VariableExpr(ident) => ident.format(fmt),
            Expression::ElementAccess(access) => access.format(fmt),
            Expression::FuncCall(func_call) => func_call.format(fmt),
            Expression::SubExpr(expr) => {
                fmt.write_all(b"(")?;
                expr.format(fmt)?;
                fmt.write_all(b")")?;
                Ok(())
            }
            Expression::Conditional(cond) => cond.format(fmt),
            Expression::Operation(op) => op.format(fmt),
            Expression::ForExpr(expr) => expr.format(fmt),
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
            Value::Array(array) => format_array(fmt, array.iter()),
            Value::Object(object) => format_object(fmt, object.iter()),
        }
    }
}

impl private::Sealed for Number {}

impl Format for Number {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_string_fragment(&self.to_string())?;
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
            ObjectKey::Expression(expr) => expr.format(fmt),
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

impl private::Sealed for ElementAccess {}

impl Format for ElementAccess {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        self.expr.format(fmt)?;
        self.operator.format(fmt)?;
        Ok(())
    }
}

impl private::Sealed for ElementAccessOperator {}

impl Format for ElementAccessOperator {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            ElementAccessOperator::AttrSplat => fmt.write_all(b".*")?,
            ElementAccessOperator::FullSplat => fmt.write_all(b"[*]")?,
            ElementAccessOperator::GetAttr(ident) => {
                fmt.write_all(b".")?;
                ident.format(fmt)?;
            }
            ElementAccessOperator::LegacyIndex(index) => {
                fmt.write_all(b".")?;
                fmt.write_int(*index)?;
            }
            ElementAccessOperator::Index(expr) => {
                fmt.write_all(b"[")?;
                expr.format(fmt)?;
                fmt.write_all(b"]")?;
            }
        }

        Ok(())
    }
}

impl private::Sealed for FuncCall {}

impl Format for FuncCall {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        self.name.format(fmt)?;
        fmt.write_all(b"(")?;

        fmt.compact_mode(true);

        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                fmt.write_all(b", ")?;
            }

            arg.format(fmt)?;
        }

        fmt.compact_mode(false);

        if self.variadic {
            fmt.write_all(b"...)")?;
        } else {
            fmt.write_all(b")")?;
        }

        Ok(())
    }
}

impl private::Sealed for Conditional {}

impl Format for Conditional {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        self.cond_expr.format(fmt)?;
        fmt.write_all(b" ? ")?;
        self.true_expr.format(fmt)?;
        fmt.write_all(b" : ")?;
        self.false_expr.format(fmt)?;
        Ok(())
    }
}

impl private::Sealed for Operation {}

impl Format for Operation {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Operation::Unary(op) => op.format(fmt),
            Operation::Binary(op) => op.format(fmt),
        }
    }
}

impl private::Sealed for UnaryOp {}

impl Format for UnaryOp {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_string_fragment(self.operator.as_str())?;
        self.expr.format(fmt)
    }
}

impl private::Sealed for BinaryOp {}

impl Format for BinaryOp {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        self.lhs_expr.format(fmt)?;
        fmt.write_all(b" ")?;
        fmt.write_string_fragment(self.operator.as_str())?;
        fmt.write_all(b" ")?;
        self.rhs_expr.format(fmt)
    }
}

impl private::Sealed for ForExpr {}

impl Format for ForExpr {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            ForExpr::List(expr) => expr.format(fmt),
            ForExpr::Object(expr) => expr.format(fmt),
        }
    }
}

impl private::Sealed for ForListExpr {}

impl Format for ForListExpr {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_all(b"[")?;
        fmt.write_all(b"for ")?;
        if let Some(key) = &self.index_var {
            key.format(fmt)?;
            fmt.write_all(b", ")?;
        }
        self.value_var.format(fmt)?;
        fmt.write_all(b" in ")?;
        self.collection_expr.format(fmt)?;
        fmt.write_all(b" : ")?;
        self.element_expr.format(fmt)?;
        if let Some(cond) = &self.cond_expr {
            fmt.write_all(b" if ")?;
            cond.format(fmt)?;
        }
        fmt.write_all(b"]")?;
        Ok(())
    }
}

impl private::Sealed for ForObjectExpr {}

impl Format for ForObjectExpr {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        fmt.write_all(b"{")?;
        fmt.write_all(b"for ")?;
        if let Some(key) = &self.key_var {
            key.format(fmt)?;
            fmt.write_all(b", ")?;
        }
        self.value_var.format(fmt)?;
        fmt.write_all(b" in ")?;
        self.collection_expr.format(fmt)?;
        fmt.write_all(b" : ")?;
        self.key_expr.format(fmt)?;
        fmt.write_all(b" => ")?;
        self.value_expr.format(fmt)?;
        if self.grouping {
            fmt.write_all(b"...")?;
        }
        if let Some(cond) = &self.cond_expr {
            fmt.write_all(b" if ")?;
            cond.format(fmt)?;
        }
        fmt.write_all(b"}")?;
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

fn format_array<W, T>(fmt: &mut Formatter<W>, array: impl Iterator<Item = T>) -> Result<()>
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

fn format_object<W, K, V>(
    fmt: &mut Formatter<W>,
    object: impl Iterator<Item = (K, V)>,
) -> Result<()>
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
