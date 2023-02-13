use super::ast::{
    Array, Attribute, BinaryOp, Block, BlockBody, BlockLabel, Body, Conditional, Directive,
    Element, ElseTemplateExpr, EndforTemplateExpr, EndifTemplateExpr, Expression, ForCond,
    ForDirective, ForExpr, ForIntro, ForTemplateExpr, FuncCall, FuncSig, HeredocTemplate,
    IfDirective, IfTemplateExpr, Interpolation, Null, Object, ObjectItem, ObjectKey,
    ObjectKeyValueSeparator, ObjectValueTerminator, Structure, Template, Traversal,
    TraversalOperator, UnaryOp,
};
use super::escape::write_escaped;
use super::repr::{Decorate, Decorated};
use crate::expr::{HeredocStripMode, Variable};
use crate::template::StripMode;
use crate::{Identifier, Number};
use std::fmt::{self, Write};

pub(crate) const NO_DECOR: (&str, &str) = ("", "");
const LEADING_SPACE_DECOR: (&str, &str) = (" ", "");
const TRAILING_SPACE_DECOR: (&str, &str) = ("", " ");
const BOTH_SPACE_DECOR: (&str, &str) = (" ", " ");
const INTERPOLATION_START: &str = "${";
const DIRECTIVE_START: &str = "%{";

pub(crate) trait EncodeDecorated {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result;
}

pub(crate) trait Encode {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result;
}

pub(crate) struct EncodeState<'a> {
    buf: &'a mut dyn fmt::Write,
    escape: bool,
    input: Option<&'a str>,
}

impl<'a> EncodeState<'a> {
    pub fn new(buf: &'a mut dyn fmt::Write, input: Option<&'a str>) -> EncodeState<'a> {
        EncodeState {
            buf,
            input,
            escape: false,
        }
    }

    pub fn escaped<F>(&mut self, f: F) -> fmt::Result
    where
        F: FnOnce(&mut EncodeState) -> fmt::Result,
    {
        self.escape = true;
        let result = f(self);
        self.escape = false;
        result
    }

    pub fn with_input<F>(&mut self, f: F) -> fmt::Result
    where
        F: FnOnce(&mut EncodeState, Option<&str>) -> fmt::Result,
    {
        self.escape = true;
        let result = f(self, self.input);
        self.escape = false;
        result
    }

    pub fn escape(&self) -> bool {
        self.escape
    }
}

impl<'a> fmt::Write for EncodeState<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buf.write_str(s)
    }
}

impl<T> EncodeDecorated for Decorated<T>
where
    T: Encode,
{
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        let decor = self.decor();
        decor.encode_prefix(buf, default_decor.0)?;
        self.as_ref().encode(buf)?;
        decor.encode_suffix(buf, default_decor.1)
    }
}

impl EncodeDecorated for Expression {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            Expression::Null(v) => v.encode_decorated(buf, default_decor),
            Expression::Bool(v) => v.encode_decorated(buf, default_decor),
            Expression::Number(v) => v.encode_decorated(buf, default_decor),
            Expression::String(v) => {
                let decor = v.decor();
                decor.encode_prefix(buf, default_decor.0)?;
                buf.write_char('"')?;
                write_escaped(buf, &v)?;
                buf.write_char('"')?;
                decor.encode_suffix(buf, default_decor.1)
            }
            Expression::Array(v) => v.encode_decorated(buf, default_decor),
            Expression::Object(v) => v.encode_decorated(buf, default_decor),
            Expression::Template(v) => {
                let decor = v.decor();
                decor.encode_prefix(buf, default_decor.0)?;
                buf.write_char('"')?;
                buf.escaped(|buf| v.as_ref().encode(buf))?;
                buf.write_char('"')?;
                decor.encode_suffix(buf, default_decor.1)
            }
            Expression::HeredocTemplate(v) => v.encode_decorated(buf, default_decor),
            Expression::Parenthesis(v) => {
                let decor = v.decor();
                decor.encode_prefix(buf, default_decor.0)?;
                buf.write_char('(')?;
                v.as_ref().encode_decorated(buf, NO_DECOR)?;
                buf.write_char(')')?;
                decor.encode_suffix(buf, default_decor.1)
            }
            Expression::Variable(v) => v.encode_decorated(buf, default_decor),
            Expression::ForExpr(v) => v.encode_decorated(buf, default_decor),
            Expression::Conditional(v) => v.encode_decorated(buf, default_decor),
            Expression::FuncCall(v) => v.encode_decorated(buf, default_decor),
            Expression::UnaryOp(v) => v.encode_decorated(buf, default_decor),
            Expression::BinaryOp(v) => v.encode_decorated(buf, default_decor),
            Expression::Traversal(v) => v.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for Null {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str("null")
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for Number {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for String {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('"')?;
        write_escaped(buf, &self)?;
        buf.write_char('"')
    }
}

impl Encode for Identifier {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for Array {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('[')?;

        for (i, value) in self.values().iter().enumerate() {
            let value_decor = if i == 0 {
                NO_DECOR
            } else {
                buf.write_char(',')?;
                LEADING_SPACE_DECOR
            };
            value.encode_decorated(buf, value_decor)?;
        }

        if self.trailing_comma() {
            buf.write_char(',')?;
        }

        self.trailing().encode_with_default(buf, "")?;
        buf.write_char(']')
    }
}

impl Encode for Object {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('{')?;

        for item in self.items().iter() {
            item.encode(buf)?;
        }

        self.trailing().encode_with_default(buf, "")?;
        buf.write_char('}')
    }
}

impl Encode for ObjectItem {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.key().encode_decorated(buf, TRAILING_SPACE_DECOR)?;

        match self.key_value_separator() {
            ObjectKeyValueSeparator::Colon => buf.write_char(':')?,
            ObjectKeyValueSeparator::Equals => buf.write_char('=')?,
        }

        self.value().encode_decorated(buf, LEADING_SPACE_DECOR)?;

        match self.value_terminator() {
            ObjectValueTerminator::Comma => buf.write_char(',')?,
            ObjectValueTerminator::Newline => buf.write_char('\n')?,
            ObjectValueTerminator::None => {}
        }

        Ok(())
    }
}

impl EncodeDecorated for ObjectKey {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            ObjectKey::Identifier(ident) => ident.encode_decorated(buf, default_decor),
            ObjectKey::Expression(expr) => expr.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for Template {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        for element in self.elements().iter() {
            element.encode(buf)?;
        }

        Ok(())
    }
}

impl Encode for Element {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self {
            Element::Literal(lit) => {
                if buf.escape() {
                    write_escaped(buf, &lit)
                } else {
                    buf.write_str(&lit)
                }
            }
            Element::Interpolation(interp) => interp.encode(buf),
            Element::Directive(dir) => dir.encode(buf),
        }
    }
}

impl Encode for Interpolation {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, INTERPOLATION_START, self.strip(), |buf| {
            self.expr().encode_decorated(buf, BOTH_SPACE_DECOR)
        })
    }
}

impl Encode for Directive {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self {
            Directive::If(dir) => dir.encode(buf),
            Directive::For(dir) => dir.encode(buf),
        }
    }
}

impl Encode for IfDirective {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.if_expr().encode(buf)?;
        if let Some(else_expr) = self.else_expr() {
            else_expr.encode(buf)?;
        }
        self.endif_expr().encode(buf)
    }
}

impl Encode for IfTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip(), |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("if")?;
            self.cond_expr().encode_decorated(buf, TRAILING_SPACE_DECOR)
        })?;
        self.template().encode(buf)
    }
}

impl Encode for ElseTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip(), |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("else")?;
            self.trailing().encode_with_default(buf, " ")
        })?;
        self.template().encode(buf)
    }
}

impl Encode for EndifTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip(), |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("endif")?;
            self.trailing().encode_with_default(buf, " ")
        })
    }
}

impl Encode for ForDirective {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.for_expr().encode(buf)?;
        self.endfor_expr().encode(buf)
    }
}

impl Encode for ForTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip(), |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("for")?;

            if let Some(key_var) = self.key_var() {
                key_var.encode_decorated(buf, LEADING_SPACE_DECOR)?;
                buf.write_char(',')?;
            }

            self.value_var().encode_decorated(buf, BOTH_SPACE_DECOR)
        })?;
        self.template().encode(buf)
    }
}

impl Encode for EndforTemplateExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        encode_strip(buf, DIRECTIVE_START, self.strip(), |buf| {
            self.preamble().encode_with_default(buf, " ")?;
            buf.write_str("endfor")?;
            self.trailing().encode_with_default(buf, " ")
        })
    }
}

impl Encode for HeredocTemplate {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self.strip() {
            HeredocStripMode::None => buf.write_str("<<")?,
            HeredocStripMode::Indent => buf.write_str("<<-")?,
        }

        write!(buf, "{}\n", self.delimiter().as_str())?;
        self.template().encode(buf)?;
        write!(buf, "{}", self.delimiter().as_str())
    }
}

impl Encode for Variable {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for ForExpr {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        if let Some(key_expr) = self.key_expr() {
            // object expr
            buf.write_char('{')?;
            self.intro().encode_decorated(buf, NO_DECOR)?;
            key_expr.encode_decorated(buf, BOTH_SPACE_DECOR)?;
            buf.write_str("=>")?;
            self.value_expr()
                .encode_decorated(buf, LEADING_SPACE_DECOR)?;
            if self.grouping() {
                buf.write_str("...")?;
            }
            if let Some(cond) = self.cond() {
                cond.encode_decorated(buf, LEADING_SPACE_DECOR)?;
            }
            buf.write_char('}')
        } else {
            // list expr
            buf.write_char('[')?;
            self.intro().encode_decorated(buf, NO_DECOR)?;
            self.value_expr()
                .encode_decorated(buf, LEADING_SPACE_DECOR)?;
            if let Some(cond) = self.cond() {
                cond.encode_decorated(buf, LEADING_SPACE_DECOR)?;
            }
            buf.write_char(']')
        }
    }
}

impl Encode for ForIntro {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str("for")?;
        if let Some(key_var) = self.key_var() {
            key_var.encode_decorated(buf, LEADING_SPACE_DECOR)?;
            buf.write_char(',')?;
        }
        self.value_var()
            .encode_decorated(buf, LEADING_SPACE_DECOR)?;
        buf.write_char(':')
    }
}

impl Encode for ForCond {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str("if")?;
        self.expr().encode_decorated(buf, LEADING_SPACE_DECOR)
    }
}

impl Encode for Conditional {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.cond_expr()
            .encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        buf.write_char('?')?;
        self.true_expr().encode_decorated(buf, BOTH_SPACE_DECOR)?;
        buf.write_char(':')?;
        self.false_expr().encode_decorated(buf, LEADING_SPACE_DECOR)
    }
}

impl Encode for FuncCall {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.name().encode_decorated(buf, NO_DECOR)?;
        self.signature().encode_decorated(buf, NO_DECOR)
    }
}

impl Encode for FuncSig {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('(')?;

        for (i, arg) in self.args().iter().enumerate() {
            let arg_decor = if i == 0 {
                NO_DECOR
            } else {
                buf.write_char(',')?;
                LEADING_SPACE_DECOR
            };

            arg.encode_decorated(buf, arg_decor)?;
        }

        if self.args().len() > 0 {
            if self.expand_final() {
                buf.write_str("...")?;
            } else if self.trailing_comma() {
                buf.write_char(',')?;
            }
        }

        self.trailing().encode_with_default(buf, "")?;
        buf.write_char(')')
    }
}

impl Encode for UnaryOp {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str(self.operator().as_str())?;
        self.expr().encode_decorated(buf, NO_DECOR)
    }
}

impl Encode for BinaryOp {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.lhs_expr()
            .encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        buf.write_str(self.operator().as_str())?;
        self.rhs_expr().encode_decorated(buf, LEADING_SPACE_DECOR)
    }
}

impl Encode for Traversal {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.expr().encode_decorated(buf, NO_DECOR)?;

        for operator in self.operators().iter() {
            operator.encode_decorated(buf, NO_DECOR)?;
        }

        Ok(())
    }
}

impl Encode for TraversalOperator {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self {
            TraversalOperator::FullSplat(_) | TraversalOperator::Index(_) => buf.write_char('[')?,
            _other => buf.write_char('.')?,
        }

        match self {
            TraversalOperator::AttrSplat(splat) | TraversalOperator::FullSplat(splat) => {
                let decor = splat.decor();
                decor.encode_prefix(buf, "")?;
                buf.write_char('*')?;
                decor.encode_suffix(buf, "")?;
            }
            TraversalOperator::GetAttr(ident) => ident.encode_decorated(buf, NO_DECOR)?,
            TraversalOperator::Index(expr) => expr.encode_decorated(buf, NO_DECOR)?,
            TraversalOperator::LegacyIndex(index) => index.encode_decorated(buf, NO_DECOR)?,
        }

        match self {
            TraversalOperator::FullSplat(_) | TraversalOperator::Index(_) => buf.write_char(']'),
            _other => Ok(()),
        }
    }
}

impl Encode for Body {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        for structure in self.structures() {
            structure.encode_decorated(buf, NO_DECOR)?;
            buf.write_char('\n')?;
        }

        Ok(())
    }
}

impl EncodeDecorated for Structure {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            Structure::Attribute(attr) => attr.encode_decorated(buf, default_decor),
            Structure::Block(block) => block.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for Attribute {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.key().encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        buf.write_char('=')?;
        self.expr().encode_decorated(buf, LEADING_SPACE_DECOR)
    }
}

impl Encode for Block {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        self.ident().encode_decorated(buf, TRAILING_SPACE_DECOR)?;

        for label in self.labels().iter() {
            label.encode_decorated(buf, TRAILING_SPACE_DECOR)?;
        }

        self.body().encode(buf)
    }
}

impl EncodeDecorated for BlockLabel {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            BlockLabel::String(string) => {
                let decor = string.decor();
                decor.encode_prefix(buf, default_decor.0)?;
                buf.write_char('"')?;
                write_escaped(buf, &string)?;
                buf.write_char('"')?;
                decor.encode_suffix(buf, default_decor.1)
            }
            BlockLabel::Identifier(ident) => ident.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for BlockBody {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('{')?;

        match self {
            BlockBody::Multiline(body) => {
                let decor = body.decor();
                decor.encode_prefix(buf, "")?;
                buf.write_char('\n')?;
                body.as_ref().encode(buf)?;
                decor.encode_suffix(buf, "")?;
            }
            BlockBody::Oneline(attr) => attr.encode_decorated(buf, BOTH_SPACE_DECOR)?,
            BlockBody::Empty(raw) => raw.encode_with_default(buf, "")?,
        }

        buf.write_char('}')
    }
}

fn encode_strip<F>(buf: &mut EncodeState, start_marker: &str, strip: StripMode, f: F) -> fmt::Result
where
    F: FnOnce(&mut EncodeState) -> fmt::Result,
{
    buf.write_str(start_marker)?;
    if strip.strip_start() {
        buf.write_char('~')?;
    }

    f(buf)?;

    if strip.strip_end() {
        buf.write_char('~')?;
    }

    buf.write_char('}')
}
