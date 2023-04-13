//! Types to represent the HCL expression sub-language.

#![allow(missing_docs)]

mod array;
mod conditional;
mod for_expr;
mod func_call;
mod object;
mod operation;
mod traversal;

pub use self::array::{Array, IntoIter, Iter, IterMut};
pub use self::conditional::Conditional;
pub use self::for_expr::{ForCond, ForExpr, ForIntro};
pub use self::func_call::{FuncArgs, FuncCall};
pub use self::object::{
    Object, ObjectIntoIter, ObjectIter, ObjectIterMut, ObjectKey, ObjectKeyMut, ObjectValue,
    ObjectValueAssignment, ObjectValueTerminator,
};
pub use self::operation::{BinaryOp, BinaryOperator, UnaryOp, UnaryOperator};
pub use self::traversal::{Splat, Traversal, TraversalOperator};
use crate::encode::{EncodeDecorated, EncodeState, NO_DECOR};
use crate::repr::{Decor, Decorate, Decorated, Formatted, SetSpan, Span};
use crate::template::{HeredocTemplate, StringTemplate};
use crate::{parser, Ident, Number};
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Null(Decorated<Null>),
    Bool(Decorated<bool>),
    Number(Formatted<Number>),
    String(Decorated<String>),
    Array(Array),
    Object(Object),
    Template(StringTemplate),
    HeredocTemplate(Box<HeredocTemplate>),
    Parenthesis(Box<Parenthesis>),
    Variable(Decorated<Ident>),
    Conditional(Box<Conditional>),
    FuncCall(Box<FuncCall>),
    Traversal(Box<Traversal>),
    UnaryOp(Box<UnaryOp>),
    BinaryOp(Box<BinaryOp>),
    ForExpr(Box<ForExpr>),
}

impl Expression {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Expression::Null(n) => n.decor_mut().despan(input),
            Expression::Bool(b) => b.decor_mut().despan(input),
            Expression::Number(n) => n.decor_mut().despan(input),
            Expression::String(s) => s.decor_mut().despan(input),
            Expression::Array(array) => array.despan(input),
            Expression::Object(object) => object.despan(input),
            Expression::Template(template) => template.despan(input),
            Expression::HeredocTemplate(heredoc) => heredoc.despan(input),
            Expression::Parenthesis(expr) => expr.despan(input),
            Expression::Variable(var) => var.decor_mut().despan(input),
            Expression::ForExpr(expr) => expr.despan(input),
            Expression::Conditional(cond) => cond.despan(input),
            Expression::FuncCall(call) => call.despan(input),
            Expression::UnaryOp(op) => op.despan(input),
            Expression::BinaryOp(op) => op.despan(input),
            Expression::Traversal(traversal) => traversal.despan(input),
        }
    }
}

impl FromStr for Expression {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_expr(s)
    }
}

impl From<&str> for Expression {
    fn from(s: &str) -> Self {
        Expression::from(String::from(s))
    }
}

impl From<String> for Expression {
    fn from(s: String) -> Self {
        Expression::from(Decorated::new(s))
    }
}

impl From<Decorated<String>> for Expression {
    fn from(s: Decorated<String>) -> Self {
        Expression::String(s)
    }
}

impl From<Array> for Expression {
    fn from(array: Array) -> Self {
        Expression::Array(array)
    }
}

impl From<Object> for Expression {
    fn from(object: Object) -> Self {
        Expression::Object(object)
    }
}

impl From<Parenthesis> for Expression {
    fn from(value: Parenthesis) -> Self {
        Expression::Parenthesis(Box::new(value))
    }
}

impl From<Traversal> for Expression {
    fn from(traversal: Traversal) -> Self {
        Expression::Traversal(Box::new(traversal))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Parenthesis {
    inner: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Parenthesis {
    pub fn new(inner: Expression) -> Parenthesis {
        Parenthesis {
            inner,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn inner(&self) -> &Expression {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Expression {
        &mut self.inner
    }

    pub fn into_inner(self) -> Expression {
        self.inner
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.inner.despan(input);
    }
}

impl PartialEq for Parenthesis {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Null;

impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "null")
    }
}

decorate_impl!(Parenthesis);
span_impl!(Parenthesis);

forward_decorate_impl!(Expression => {
    Null, Bool, Number, String, Array, Object, Template, HeredocTemplate, Parenthesis,
    Variable, ForExpr, Conditional, FuncCall, UnaryOp, BinaryOp, Traversal
});
forward_span_impl!(Expression => {
    Null, Bool, Number, String, Array, Object, Template, HeredocTemplate, Parenthesis,
    Variable, ForExpr, Conditional, FuncCall, UnaryOp, BinaryOp, Traversal
});
