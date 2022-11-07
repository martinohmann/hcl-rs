use super::{private, Format, Formatter};
use crate::expr::{
    BinaryOp, Conditional, Expression, ForExpr, FuncCall, Heredoc, HeredocStripMode, ObjectKey,
    Operation, RawExpression, TemplateExpr, Traversal, TraversalOperator, UnaryOp,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use crate::template::{
    Directive, Element, ForDirective, IfDirective, Interpolation, StripMode, Template,
};
use crate::{Identifier, Number, Result, Value};
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
        self.key.format(fmt)?;
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
        self.identifier.format(fmt)?;

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
            Expression::Variable(ident) => ident.format(fmt),
            Expression::Traversal(traversal) => traversal.format(fmt),
            Expression::FuncCall(func_call) => func_call.format(fmt),
            Expression::Parenthesis(expr) => {
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
        fmt.write_string_fragment(self.strip.as_str())?;
        fmt.write_string_fragment(&self.delimiter)?;
        fmt.write_all(b"\n")?;
        fmt.write_string_fragment(&self.template)?;

        if !self.template.ends_with('\n') {
            fmt.write_all(b"\n")?;
        }

        match self.strip {
            HeredocStripMode::None => {
                fmt.write_string_fragment(&self.delimiter)?;
            }
            HeredocStripMode::Indent => {
                fmt.write_indented(fmt.current_indent, &self.delimiter)?;
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
        fmt.write_string_fragment(self)?;
        Ok(())
    }
}

impl private::Sealed for Traversal {}

impl Format for Traversal {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        self.expr.format(fmt)?;
        for operator in &self.operators {
            operator.format(fmt)?;
        }
        Ok(())
    }
}

impl private::Sealed for TraversalOperator {}

impl Format for TraversalOperator {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            TraversalOperator::AttrSplat => fmt.write_all(b".*")?,
            TraversalOperator::FullSplat => fmt.write_all(b"[*]")?,
            TraversalOperator::GetAttr(ident) => {
                fmt.write_all(b".")?;
                ident.format(fmt)?;
            }
            TraversalOperator::LegacyIndex(index) => {
                fmt.write_all(b".")?;
                fmt.write_int(*index)?;
            }
            TraversalOperator::Index(expr) => {
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

        fmt.with_compact_mode(|fmt| {
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    fmt.write_all(b", ")?;
                }

                arg.format(fmt)?;
            }

            Ok(())
        })?;

        if self.expand_final {
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
        let object_result = self.key_expr.is_some();

        if object_result {
            fmt.write_all(b"{")?;
        } else {
            fmt.write_all(b"[")?;
        }

        fmt.write_all(b"for ")?;
        if let Some(key) = &self.key_var {
            key.format(fmt)?;
            fmt.write_all(b", ")?;
        }
        self.value_var.format(fmt)?;
        fmt.write_all(b" in ")?;
        self.collection_expr.format(fmt)?;
        fmt.write_all(b" : ")?;

        if let Some(key_expr) = &self.key_expr {
            key_expr.format(fmt)?;
            fmt.write_all(b" => ")?;
        }
        self.value_expr.format(fmt)?;
        if object_result && self.grouping {
            fmt.write_all(b"...")?;
        }
        if let Some(cond) = &self.cond_expr {
            fmt.write_all(b" if ")?;
            cond.format(fmt)?;
        }

        if object_result {
            fmt.write_all(b"}")?;
        } else {
            fmt.write_all(b"]")?;
        }
        Ok(())
    }
}

impl private::Sealed for Template {}

impl Format for Template {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        for element in self.elements() {
            element.format(fmt)?;
        }
        Ok(())
    }
}

impl private::Sealed for Element {}

impl Format for Element {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Element::Literal(lit) => {
                fmt.write_string_fragment(lit)?;
                Ok(())
            }
            Element::Interpolation(interp) => interp.format(fmt),
            Element::Directive(dir) => dir.format(fmt),
        }
    }
}

impl private::Sealed for Interpolation {}

impl Format for Interpolation {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        format_interpolation(fmt, self.strip, |fmt| self.expr.format(fmt))
    }
}

impl private::Sealed for Directive {}

impl Format for Directive {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        match self {
            Directive::If(if_dir) => if_dir.format(fmt),
            Directive::For(for_dir) => for_dir.format(fmt),
        }
    }
}

impl private::Sealed for IfDirective {}

impl Format for IfDirective {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        format_directive(fmt, self.if_strip, |fmt| {
            fmt.write_all(b"if")?;
            self.cond_expr.format(fmt)
        })?;

        self.true_template.format(fmt)?;

        if let Some(false_template) = &self.false_template {
            format_directive(fmt, self.else_strip, |fmt| {
                fmt.write_all(b"else")?;
                Ok(())
            })?;

            false_template.format(fmt)?;
        }

        format_directive(fmt, self.endif_strip, |fmt| {
            fmt.write_all(b"endif")?;
            Ok(())
        })
    }
}

impl private::Sealed for ForDirective {}

impl Format for ForDirective {
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write,
    {
        format_directive(fmt, self.for_strip, |fmt| {
            fmt.write_all(b"for ")?;
            if let Some(key_var) = &self.key_var {
                key_var.format(fmt)?;
                fmt.write_all(b", ")?;
            }
            self.value_var.format(fmt)?;
            fmt.write_all(b" in ")?;
            self.collection_expr.format(fmt)
        })?;

        self.template.format(fmt)?;

        format_directive(fmt, self.endfor_strip, |fmt| {
            fmt.write_all(b"endfor")?;
            Ok(())
        })
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

fn format_strip<W, F>(fmt: &mut Formatter<W>, strip: StripMode, f: F) -> Result<()>
where
    W: io::Write,
    F: FnOnce(&mut Formatter<W>) -> Result<()>,
{
    if strip.strip_start() {
        fmt.write_all(b"~")?;
    }

    f(fmt)?;

    if strip.strip_end() {
        fmt.write_all(b"~")?;
    }

    Ok(())
}

fn format_interpolation<W, F>(fmt: &mut Formatter<W>, strip: StripMode, f: F) -> Result<()>
where
    W: io::Write,
    F: FnOnce(&mut Formatter<W>) -> Result<()>,
{
    fmt.write_all(b"${")?;
    format_strip(fmt, strip, f)?;
    fmt.write_all(b"}")?;
    Ok(())
}

fn format_directive<W, F>(fmt: &mut Formatter<W>, strip: StripMode, f: F) -> Result<()>
where
    W: io::Write,
    F: FnOnce(&mut Formatter<W>) -> Result<()>,
{
    fmt.write_all(b"%{")?;
    format_strip(fmt, strip, |fmt| {
        fmt.write_all(b" ")?;
        f(fmt)?;
        fmt.write_all(b" ")?;
        Ok(())
    })?;
    fmt.write_all(b"}")?;
    Ok(())
}
