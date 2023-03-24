use super::{
    encode_decorated, encode_quoted_string, Encode, EncodeDecorated, EncodeState, BOTH_SPACE_DECOR,
    LEADING_SPACE_DECOR, NO_DECOR, TRAILING_SPACE_DECOR,
};
use crate::expr::{
    Array, BinaryOp, Conditional, Expression, ForCond, ForExpr, ForIntro, FuncArgs, FuncCall, Null,
    Object, ObjectKey, ObjectValue, ObjectValueAssignment, ObjectValueTerminator, Parenthesis,
    Splat, Traversal, TraversalOperator, UnaryOp,
};
use std::fmt::{self, Write};

impl Encode for Null {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{self}")
    }
}

impl Encode for Splat {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{self}")
    }
}

impl EncodeDecorated for Expression {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            Expression::Null(v) => v.encode_decorated(buf, default_decor),
            Expression::Bool(v) => v.encode_decorated(buf, default_decor),
            Expression::Number(v) => v.encode_decorated(buf, default_decor),
            Expression::String(v) => {
                encode_decorated(v, buf, default_decor, |buf| encode_quoted_string(buf, v))
            }
            Expression::Array(v) => v.encode_decorated(buf, default_decor),
            Expression::Object(v) => v.encode_decorated(buf, default_decor),
            Expression::Template(v) => v.encode_decorated(buf, default_decor),
            Expression::HeredocTemplate(v) => v.encode_decorated(buf, default_decor),
            Expression::Parenthesis(v) => v.encode_decorated(buf, default_decor),
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

impl Encode for Parenthesis {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('(')?;
        self.inner().encode_decorated(buf, NO_DECOR)?;
        buf.write_char(')')
    }
}

impl Encode for Array {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('[')?;

        if !self.is_empty() {
            for (i, value) in self.iter().enumerate() {
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
        }

        self.trailing().encode_with_default(buf, "")?;
        buf.write_char(']')
    }
}

impl Encode for Object {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('{')?;

        for (key, value) in self.iter() {
            key.encode_decorated(buf, TRAILING_SPACE_DECOR)?;
            value.encode(buf)?;
        }

        self.trailing().encode_with_default(buf, "")?;
        buf.write_char('}')
    }
}

impl EncodeDecorated for ObjectKey {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        match self {
            ObjectKey::Ident(ident) => ident.encode_decorated(buf, default_decor),
            ObjectKey::Expression(expr) => expr.encode_decorated(buf, default_decor),
        }
    }
}

impl Encode for ObjectValue {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        match self.assignment() {
            ObjectValueAssignment::Colon => buf.write_char(':')?,
            ObjectValueAssignment::Equals => buf.write_char('=')?,
        }

        self.expr().encode_decorated(buf, LEADING_SPACE_DECOR)?;

        match self.terminator() {
            ObjectValueTerminator::Comma => buf.write_char(',')?,
            ObjectValueTerminator::Newline => buf.write_char('\n')?,
            ObjectValueTerminator::None => {}
        }

        Ok(())
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
        buf.write_str("in")?;
        self.collection_expr()
            .encode_decorated(buf, BOTH_SPACE_DECOR)?;
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
        self.args().encode_decorated(buf, NO_DECOR)
    }
}

impl Encode for FuncArgs {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_char('(')?;

        for (i, arg) in self.iter().enumerate() {
            let arg_decor = if i == 0 {
                NO_DECOR
            } else {
                buf.write_char(',')?;
                LEADING_SPACE_DECOR
            };

            arg.encode_decorated(buf, arg_decor)?;
        }

        if !self.is_empty() {
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

        for operator in self.operators() {
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
                splat.encode_decorated(buf, NO_DECOR)?
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
