use super::ast::{
    Array, BinaryOp, Conditional, Expression, ForExpr, FuncCall, HeredocTemplate, Null, Object,
    Template, Traversal, TraversalOperator, UnaryOp,
};
use super::repr::{Decorate, Decorated};
use crate::expr::{BinaryOperator, UnaryOperator, Variable};
use crate::{Identifier, Number};
use std::fmt;

pub(crate) trait Encode {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result;
}

impl<T> Encode for Decorated<T>
where
    T: Encode,
{
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        let decor = self.decor();
        decor.encode_prefix(buf, input, default_decor.0)?;
        (&**self).encode(buf, input, default_decor)?;
        decor.encode_suffix(buf, input, default_decor.1)
    }
}

impl Encode for Expression {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            Expression::Null(n) => n.encode(buf, input, default_decor),
            Expression::Bool(b) => b.encode(buf, input, default_decor),
            Expression::Number(n) => n.encode(buf, input, default_decor),
            Expression::String(s) => s.encode(buf, input, default_decor),
            Expression::Array(array) => array.encode(buf, input, default_decor),
            Expression::Object(object) => object.encode(buf, input, default_decor),
            Expression::Template(template) => template.encode(buf, input, default_decor),
            Expression::HeredocTemplate(heredoc) => heredoc.encode(buf, input, default_decor),
            Expression::Parenthesis(expr) => expr.encode(buf, input, default_decor),
            Expression::Variable(var) => var.encode(buf, input, default_decor),
            Expression::ForExpr(expr) => expr.encode(buf, input, default_decor),
            Expression::Conditional(cond) => cond.encode(buf, input, default_decor),
            Expression::FuncCall(call) => call.encode(buf, input, default_decor),
            Expression::UnaryOp(op) => op.encode(buf, input, default_decor),
            Expression::BinaryOp(op) => op.encode(buf, input, default_decor),
            Expression::Traversal(traversal) => traversal.encode(buf, input, default_decor),
        }
    }
}

impl Encode for Null {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_str("null")
    }
}

impl Encode for bool {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for u64 {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for Number {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for String {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        write!(buf, "{}", self)
    }
}

impl Encode for Identifier {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for Array {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_char('[')?;

        for (i, value) in self.values().iter().enumerate() {
            let default_decor = if i == 0 {
                ("", "")
            } else {
                buf.write_char(',')?;
                (" ", "")
            };
            value.encode(buf, input, default_decor)?;
        }

        if self.trailing_comma() {
            buf.write_char(',')?;
        }

        self.trailing().encode_with_default(buf, input, "")?;
        buf.write_char(']')
    }
}

impl Encode for Object {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_char('{')?;
        unimplemented!();
        self.trailing().encode_with_default(buf, input, "")?;
        buf.write_char('}')
    }
}

impl Encode for Template {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        unimplemented!();
    }
}

impl Encode for HeredocTemplate {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        unimplemented!();
    }
}

impl Encode for Variable {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        unimplemented!();
    }
}

impl Encode for ForExpr {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        unimplemented!();
    }
}

impl Encode for Conditional {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        self.cond_expr()
            .encode(buf, input, (default_decor.0, " "))?;
        buf.write_char('?')?;
        self.true_expr().encode(buf, input, (" ", " "))?;
        buf.write_char(':')?;
        self.false_expr().encode(buf, input, (" ", default_decor.1))
    }
}

impl Encode for FuncCall {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        self.name().encode(buf, input, (default_decor.0, ""))?;
        buf.write_char('(')?;
        for (i, arg) in self.args().iter().enumerate() {
            let default_decor = if i == 0 {
                ("", "")
            } else {
                buf.write_char(',')?;
                (" ", "")
            };
            arg.encode(buf, input, default_decor)?;
        }

        // @TODO(mohmann): handle trailing whitespace after ellipsis.
        if self.expand_final() {
            buf.write_str("...")?;
        }

        buf.write_char(')')
    }
}

impl Encode for UnaryOp {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        self.operator().encode(buf, input, (default_decor.0, ""))?;
        self.expr().encode(buf, input, ("", default_decor.1))
    }
}

impl Encode for BinaryOp {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        self.lhs_expr().encode(buf, input, (default_decor.0, " "))?;
        self.operator().encode(buf, input, ("", ""))?;
        self.rhs_expr().encode(buf, input, (" ", default_decor.1))
    }
}

impl Encode for Traversal {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        default_decor: (&str, &str),
    ) -> fmt::Result {
        let operators = self.operators();

        if operators.is_empty() {
            self.expr().encode(buf, input, default_decor)
        } else {
            self.expr().encode(buf, input, (default_decor.0, ""))?;

            for (i, operator) in self.operators().iter().enumerate() {
                let op_decor = if i < operators.len() - 1 {
                    ("", "")
                } else {
                    ("", default_decor.1)
                };
                operator.encode(buf, input, op_decor)?;
            }

            Ok(())
        }
    }
}

impl Encode for UnaryOperator {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for BinaryOperator {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        _input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

impl Encode for TraversalOperator {
    fn encode(
        &self,
        buf: &mut dyn fmt::Write,
        input: Option<&str>,
        _default_decor: (&str, &str),
    ) -> fmt::Result {
        match self {
            TraversalOperator::AttrSplat | TraversalOperator::LegacyIndex(_) => {
                buf.write_char('.')?
            }
            _other => buf.write_char('[')?,
        }

        // @TODO(mohmann): handle whitespace within splat operators.
        match self {
            TraversalOperator::AttrSplat | TraversalOperator::FullSplat => buf.write_char('*')?,
            TraversalOperator::GetAttr(ident) => ident.encode(buf, input, ("", ""))?,
            TraversalOperator::Index(expr) => expr.encode(buf, input, ("", ""))?,
            TraversalOperator::LegacyIndex(index) => index.encode(buf, input, ("", ""))?,
        }

        match self {
            TraversalOperator::AttrSplat | TraversalOperator::LegacyIndex(_) => Ok(()),
            _other => buf.write_char(']'),
        }
    }
}
