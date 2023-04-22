//! Types to represent the HCL expression sub-language.

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

/// A type representing any expression from the expression sub-language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Represents a null value.
    Null(Decorated<Null>),
    /// Represents a boolean.
    Bool(Decorated<bool>),
    /// Represents a number, either integer or float.
    Number(Formatted<Number>),
    /// Represents a string that does not contain any template interpolations or template
    /// directives.
    String(Decorated<String>),
    /// Represents an HCL array.
    Array(Array),
    /// Represents an HCL object.
    Object(Object),
    /// Represents a string containing template interpolations and template directives.
    Template(StringTemplate),
    /// Represents an HCL heredoc template.
    HeredocTemplate(Box<HeredocTemplate>),
    /// Represents a sub-expression wrapped in parenthesis.
    Parenthesis(Box<Parenthesis>),
    /// Represents a variable identifier.
    Variable(Decorated<Ident>),
    /// Represents conditional operator which selects one of two rexpressions based on the outcome
    /// of a boolean expression.
    Conditional(Box<Conditional>),
    /// Represents a function call.
    FuncCall(Box<FuncCall>),
    /// Represents an attribute or element traversal.
    Traversal(Box<Traversal>),
    /// Represents an operation which applies a unary operator to an expression.
    UnaryOp(Box<UnaryOp>),
    /// Represents an operation which applies a binary operator to two expressions.
    BinaryOp(Box<BinaryOp>),
    /// Represents a construct for constructing a collection by projecting the items from another
    /// collection.
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

impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Expression::from(Decorated::new(value))
    }
}

impl From<Decorated<bool>> for Expression {
    fn from(value: Decorated<bool>) -> Self {
        Expression::Bool(value)
    }
}

impl From<Number> for Expression {
    fn from(value: Number) -> Self {
        Expression::from(Formatted::new(value))
    }
}

impl From<Formatted<Number>> for Expression {
    fn from(value: Formatted<Number>) -> Self {
        Expression::Number(value)
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        Expression::from(String::from(value))
    }
}

impl From<String> for Expression {
    fn from(value: String) -> Self {
        Expression::from(Decorated::new(value))
    }
}

impl From<Decorated<String>> for Expression {
    fn from(value: Decorated<String>) -> Self {
        Expression::String(value)
    }
}

impl From<Array> for Expression {
    fn from(value: Array) -> Self {
        Expression::Array(value)
    }
}

impl From<Object> for Expression {
    fn from(value: Object) -> Self {
        Expression::Object(value)
    }
}

impl From<StringTemplate> for Expression {
    fn from(value: StringTemplate) -> Self {
        Expression::Template(value)
    }
}

impl From<HeredocTemplate> for Expression {
    fn from(value: HeredocTemplate) -> Self {
        Expression::HeredocTemplate(Box::new(value))
    }
}

impl From<Parenthesis> for Expression {
    fn from(value: Parenthesis) -> Self {
        Expression::Parenthesis(Box::new(value))
    }
}

impl From<Ident> for Expression {
    fn from(value: Ident) -> Self {
        Expression::from(Decorated::new(value))
    }
}

impl From<Decorated<Ident>> for Expression {
    fn from(value: Decorated<Ident>) -> Self {
        Expression::Variable(value)
    }
}

impl From<Conditional> for Expression {
    fn from(value: Conditional) -> Self {
        Expression::Conditional(Box::new(value))
    }
}

impl From<FuncCall> for Expression {
    fn from(value: FuncCall) -> Self {
        Expression::FuncCall(Box::new(value))
    }
}

impl From<Traversal> for Expression {
    fn from(value: Traversal) -> Self {
        Expression::Traversal(Box::new(value))
    }
}

impl From<UnaryOp> for Expression {
    fn from(value: UnaryOp) -> Self {
        Expression::UnaryOp(Box::new(value))
    }
}

impl From<BinaryOp> for Expression {
    fn from(value: BinaryOp) -> Self {
        Expression::BinaryOp(Box::new(value))
    }
}

impl From<ForExpr> for Expression {
    fn from(value: ForExpr) -> Self {
        Expression::ForExpr(Box::new(value))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode_decorated(&mut state, NO_DECOR)
    }
}

/// Represents a sub-expression wrapped in parenthesis (`( <expr> )`).
#[derive(Debug, Clone, Eq)]
pub struct Parenthesis {
    inner: Expression,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl Parenthesis {
    /// Creates a new `Parenthesis` value from an `Expression`.
    pub fn new(inner: Expression) -> Parenthesis {
        Parenthesis {
            inner,
            decor: Decor::default(),
            span: None,
        }
    }

    /// Returns a reference to the wrapped `Expression`.
    pub fn inner(&self) -> &Expression {
        &self.inner
    }

    /// Returns a mutable reference to the wrapped `Expression`.
    pub fn inner_mut(&mut self) -> &mut Expression {
        &mut self.inner
    }

    /// Consumes the `Parenthesis` and returns the wrapped `Expression`.
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

/// Represents a value that is `null`.
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
