use super::ast::{
    Array, Attribute, BinaryOp, Block, BlockBody, BlockLabel, Body, Conditional, Directive,
    Element, Expression, ForDirective, ForExpr, FuncCall, HeredocTemplate, IfDirective,
    Interpolation, Null, Object, ObjectItem, ObjectKey, ObjectKeyValueSeparator,
    ObjectValueTerminator, Structure, Template, Traversal, TraversalOperator, UnaryOp,
};
use super::repr::{Decorate, Decorated};
use crate::expr::{HeredocStripMode, Variable};
use crate::{Identifier, Number};
use std::fmt;

pub(crate) const NO_DECOR: (&str, &str) = ("", "");
const LEADING_SPACE_DECOR: (&str, &str) = (" ", "");
const TRAILING_SPACE_DECOR: (&str, &str) = ("", " ");
const BOTH_SPACE_DECOR: (&str, &str) = (" ", " ");

pub(crate) trait EncodeDecorated {
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result;
}

pub(crate) trait Encode {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result;
}

impl<T> EncodeDecorated for Decorated<T>
where
    T: Encode,
{
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        let decor = self.decor();
        decor.encode_prefix(buf, input, default_decor.0)?;
        (&**self).encode(buf, input)?;
        decor.encode_suffix(buf, input, default_decor.1)
    }
}

impl EncodeDecorated for Expression {
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            Expression::Null(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Bool(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Number(v) => v.encode_decorated(buf, input, default_decor),
            Expression::String(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Array(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Object(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Template(v) => {
                let decor = v.decor();
                decor.encode_prefix(buf, input, default_decor.0)?;
                // @FIXME(mohmann): properly escape string literals.
                buf.write_char('"')?;
                v.as_ref().encode(buf, input)?;
                buf.write_char('"')?;
                decor.encode_suffix(buf, input, default_decor.1)
            }
            Expression::HeredocTemplate(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Parenthesis(v) => {
                let decor = v.decor();
                decor.encode_prefix(buf, input, default_decor.0)?;
                buf.write_char('(')?;
                v.as_ref().encode_decorated(buf, input, NO_DECOR)?;
                buf.write_char(')')?;
                decor.encode_suffix(buf, input, default_decor.1)
            }
            Expression::Variable(v) => v.encode_decorated(buf, input, default_decor),
            Expression::ForExpr(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Conditional(v) => v.encode_decorated(buf, input, default_decor),
            Expression::FuncCall(v) => v.encode_decorated(buf, input, default_decor),
            Expression::UnaryOp(v) => v.encode_decorated(buf, input, default_decor),
            Expression::BinaryOp(v) => v.encode_decorated(buf, input, default_decor),
            Expression::Traversal(v) => v.encode_decorated(buf, input, default_decor),
        }
    }
}

impl Encode for Null {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        buf.write_str("null")
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for Number {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for String {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        // @FIXME: properly escape string.
        write!(buf, "\"{}\"", self)
    }
}

impl Encode for Identifier {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for Array {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        buf.write_char('[')?;

        for (i, value) in self.values().iter().enumerate() {
            let value_decor = if i == 0 {
                NO_DECOR
            } else {
                buf.write_char(',')?;
                LEADING_SPACE_DECOR
            };
            value.encode_decorated(buf, input, value_decor)?;
        }

        if self.trailing_comma() {
            buf.write_char(',')?;
        }

        self.trailing().encode_with_default(buf, input, "")?;
        buf.write_char(']')
    }
}

impl Encode for Object {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        buf.write_char('{')?;

        for item in self.items().iter() {
            // @TODO(mohmann): more sensible default decor.
            item.encode_decorated(buf, input, NO_DECOR)?;
        }

        self.trailing().encode_with_default(buf, input, "")?;
        buf.write_char('}')
    }
}

impl Encode for ObjectItem {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.key()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;

        match self.key_value_separator() {
            ObjectKeyValueSeparator::Colon => buf.write_char(':')?,
            ObjectKeyValueSeparator::Equals => buf.write_char('=')?,
        }

        self.value()
            .encode_decorated(buf, input, LEADING_SPACE_DECOR)?;

        match self.value_terminator() {
            ObjectValueTerminator::Comma => buf.write_char(',')?,
            ObjectValueTerminator::Newline => buf.write_char('\n')?,
            ObjectValueTerminator::None => {}
        }

        Ok(())
    }
}

impl EncodeDecorated for ObjectKey {
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            ObjectKey::Identifier(ident) => ident.encode_decorated(buf, input, default_decor),
            ObjectKey::Expression(expr) => expr.encode_decorated(buf, input, default_decor),
        }
    }
}

impl Encode for Template {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        for element in self.elements().iter() {
            element.encode(buf, input)?;
        }

        Ok(())
    }
}

impl Encode for Element {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        match self {
            Element::Literal(lit) => buf.write_str(lit.as_str()),
            Element::Interpolation(interp) => interp.encode(buf, input),
            Element::Directive(dir) => dir.encode(buf, input),
        }
    }
}

impl Encode for Interpolation {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        let strip = self.strip();
        buf.write_str("${")?;
        if strip.strip_start() {
            buf.write_char('~')?;
        }
        self.expr().encode_decorated(buf, input, BOTH_SPACE_DECOR)?;
        if strip.strip_end() {
            buf.write_char('~')?;
        }
        buf.write_char('}')
    }
}

impl Encode for Directive {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        match self {
            Directive::If(dir) => dir.encode(buf, input),
            Directive::For(dir) => dir.encode(buf, input),
        }
    }
}

impl Encode for IfDirective {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        let if_strip = self.if_strip();
        buf.write_str("%{")?;
        if if_strip.strip_start() {
            buf.write_char('~')?;
        }

        // @TODO(mohmann): leading whitespace of `if` needs to be tracked.
        buf.write_str(" if ")?;

        self.cond_expr()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;

        if if_strip.strip_end() {
            buf.write_char('~')?;
        }
        buf.write_char('}')?;
        self.true_template().encode(buf, input)?;

        if let Some(false_template) = self.false_template() {
            let else_strip = self.else_strip();
            buf.write_str("%{")?;
            if else_strip.strip_start() {
                buf.write_char('~')?;
            }

            // @TODO(mohmann): surround whitespace of `else` needs to be tracked.
            buf.write_str(" else ")?;

            if else_strip.strip_end() {
                buf.write_char('~')?;
            }
            buf.write_char('}')?;
            false_template.encode(buf, input)?;
        }

        let endif_strip = self.endif_strip();
        buf.write_str("%{")?;
        if endif_strip.strip_start() {
            buf.write_char('~')?;
        }

        // @TODO(mohmann): surrounding whitespace of `endif` needs to be tracked.
        buf.write_str(" endif ")?;

        if endif_strip.strip_end() {
            buf.write_char('~')?;
        }
        buf.write_char('}')
    }
}

impl Encode for ForDirective {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        let for_strip = self.for_strip();
        buf.write_str("%{")?;
        if for_strip.strip_start() {
            buf.write_char('~')?;
        }

        // @TODO(mohmann): leading whitespace of `for` needs to be tracked.
        buf.write_str(" for ")?;

        if let Some(key_var) = self.key_var() {
            key_var.encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            buf.write_char(',')?;
        }

        self.value_var()
            .encode_decorated(buf, input, BOTH_SPACE_DECOR)?;

        if for_strip.strip_end() {
            buf.write_char('~')?;
        }
        buf.write_char('}')?;
        self.template().encode(buf, input)?;

        let endfor_strip = self.endfor_strip();
        buf.write_str("%{")?;
        if endfor_strip.strip_start() {
            buf.write_char('~')?;
        }

        // @TODO(mohmann): surrounding whitespace of `endfor` needs to be tracked.
        buf.write_str(" endfor ")?;

        if endfor_strip.strip_end() {
            buf.write_char('~')?;
        }
        buf.write_char('}')
    }
}

impl Encode for HeredocTemplate {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        match self.strip() {
            HeredocStripMode::None => buf.write_str("<<")?,
            HeredocStripMode::Indent => buf.write_str("<<-")?,
        }

        write!(buf, "{}\n", self.delimiter().as_str())?;
        self.template().encode(buf, input)?;
        write!(buf, "{}", self.delimiter().as_str())
    }
}

impl Encode for Variable {
    fn encode(&self, buf: &mut dyn fmt::Write, _input: Option<&str>) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for ForExpr {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        if let Some(key_expr) = self.key_expr() {
            // object expr
            buf.write_char('{')?;
            self.prefix().encode_with_default(buf, input, "")?;
            buf.write_str("for")?;
            if let Some(key_var) = self.key_var() {
                key_var.encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
                buf.write_char(',')?;
            }
            self.value_var()
                .encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            buf.write_char(':')?;
            key_expr.encode_decorated(buf, input, BOTH_SPACE_DECOR)?;
            buf.write_str("=>")?;
            self.value_expr()
                .encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            if self.grouping() {
                buf.write_str("...")?;
            }
            if let Some(cond_expr) = self.cond_expr() {
                cond_expr.encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            }
            buf.write_char('}')
        } else {
            // list expr
            buf.write_char('[')?;
            self.prefix().encode_with_default(buf, input, "")?;
            buf.write_str("for")?;
            if let Some(key_var) = self.key_var() {
                key_var.encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
                buf.write_char(',')?;
            }
            self.value_var()
                .encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            buf.write_char(':')?;
            self.value_expr()
                .encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            if let Some(cond_expr) = self.cond_expr() {
                cond_expr.encode_decorated(buf, input, LEADING_SPACE_DECOR)?;
            }
            buf.write_char(']')
        }
    }
}

impl Encode for Conditional {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.cond_expr()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;
        buf.write_char('?')?;
        self.true_expr()
            .encode_decorated(buf, input, BOTH_SPACE_DECOR)?;
        buf.write_char(':')?;
        self.false_expr()
            .encode_decorated(buf, input, LEADING_SPACE_DECOR)
    }
}

impl Encode for FuncCall {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.name().encode_decorated(buf, input, NO_DECOR)?;
        buf.write_char('(')?;
        for (i, arg) in self.args().iter().enumerate() {
            let arg_decor = if i == 0 {
                NO_DECOR
            } else {
                buf.write_char(',')?;
                LEADING_SPACE_DECOR
            };
            arg.encode_decorated(buf, input, arg_decor)?;
        }

        // @TODO(mohmann): handle trailing whitespace after ellipsis.
        if self.expand_final() {
            buf.write_str("...")?;
        }

        // @FIXME(mohmann): handle trailing comma.

        buf.write_char(')')
    }
}

impl Encode for UnaryOp {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        buf.write_str(self.operator().as_str())?;
        self.expr().encode_decorated(buf, input, NO_DECOR)
    }
}

impl Encode for BinaryOp {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.lhs_expr()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;
        buf.write_str(self.operator().as_str())?;
        self.rhs_expr()
            .encode_decorated(buf, input, LEADING_SPACE_DECOR)
    }
}

impl Encode for Traversal {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.expr().encode_decorated(buf, input, NO_DECOR)?;

        for operator in self.operators().iter() {
            operator.encode_decorated(buf, input, NO_DECOR)?;
        }

        Ok(())
    }
}

impl Encode for TraversalOperator {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        match self {
            TraversalOperator::FullSplat | TraversalOperator::Index(_) => buf.write_char('[')?,
            _other => buf.write_char('.')?,
        }

        // @TODO(mohmann): handle whitespace within splat operators.
        match self {
            TraversalOperator::AttrSplat | TraversalOperator::FullSplat => buf.write_char('*')?,
            TraversalOperator::GetAttr(ident) => ident.encode_decorated(buf, input, NO_DECOR)?,
            TraversalOperator::Index(expr) => expr.encode_decorated(buf, input, NO_DECOR)?,
            TraversalOperator::LegacyIndex(index) => {
                index.encode_decorated(buf, input, NO_DECOR)?
            }
        }

        match self {
            TraversalOperator::FullSplat | TraversalOperator::Index(_) => buf.write_char(']'),
            _other => Ok(()),
        }
    }
}

impl Encode for Body {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        for structure in self.structures() {
            structure.encode_decorated(buf, input, NO_DECOR)?;
            buf.write_char('\n')?;
        }

        Ok(())
    }
}

impl EncodeDecorated for Structure {
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            Structure::Attribute(attr) => attr.encode_decorated(buf, input, default_decor),
            Structure::Block(block) => block.encode_decorated(buf, input, default_decor),
        }
    }
}

impl Encode for Attribute {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.key()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;
        buf.write_char('=')?;
        self.expr()
            .encode_decorated(buf, input, LEADING_SPACE_DECOR)
    }
}

impl Encode for Block {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        self.ident()
            .encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;

        for label in self.labels().iter() {
            label.encode_decorated(buf, input, TRAILING_SPACE_DECOR)?;
        }

        self.body().encode(buf, input)
    }
}

impl EncodeDecorated for BlockLabel {
    fn encode_decorated(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            BlockLabel::String(string) => string.encode_decorated(buf, input, default_decor),
            BlockLabel::Identifier(ident) => ident.encode_decorated(buf, input, default_decor),
        }
    }
}

impl Encode for BlockBody {
    fn encode(&self, buf: &mut dyn fmt::Write, input: Option<&str>) -> fmt::Result {
        buf.write_char('{')?;

        match self {
            BlockBody::Multiline(body) => {
                let decor = body.decor();
                decor.encode_prefix(buf, input, "")?;
                buf.write_char('\n')?;
                body.as_ref().encode(buf, input)?;
                decor.encode_suffix(buf, input, "")?;
            }
            BlockBody::Oneline(attr) => attr.encode_decorated(buf, input, BOTH_SPACE_DECOR)?,
            BlockBody::Empty(raw) => raw.encode_with_default(buf, input, "")?,
        }

        buf.write_char('}')
    }
}
