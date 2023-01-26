use crate::{BlockLabel, Identifier, Number};
use nom_locate::LocatedSpan;
use std::borrow::Cow;
use vecmap::VecMap;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Str<'a> = Cow<'a, str>;

#[derive(Default)]
pub struct Decor<'a> {
    pub prefix: Option<Str<'a>>,
    pub suffix: Option<Str<'a>>,
}

#[derive(Debug, Clone)]
pub struct Spanned<'a, T> {
    pub value: T,
    pub start: Span<'a>,
    pub end: Span<'a>,
    // pub decor: Decor<'a>,
}

impl<'a, T> Spanned<'a, T> {
    pub fn map_value<F, U>(self, f: F) -> Spanned<'a, U>
    where
        F: FnOnce(T) -> U,
    {
        Spanned {
            value: f(self.value),
            start: self.start,
            end: self.end,
        }
    }
}

pub type SpannedExpr<'a> = Spanned<'a, Expr<'a>>;
pub type SpannedNull<'a> = Spanned<'a, ()>;
pub type SpannedBool<'a> = Spanned<'a, bool>;
pub type SpannedNumber<'a> = Spanned<'a, Number>;
pub type SpannedStr<'a> = Spanned<'a, Str<'a>>;
pub type SpannedTemplate<'a> = Spanned<'a, Template<'a>>;
pub type SpannedHeredocTemplate<'a> = Spanned<'a, HeredocTemplate<'a>>;

pub enum Expr<'a> {
    Null(SpannedNull<'a>),
    Bool(SpannedBool<'a>),
    Number(SpannedNumber<'a>),
    String(SpannedStr<'a>),
    Array(Spanned<'a, Vec<Expr<'a>>>),
    Object(Spanned<'a, VecMap<ObjectKey<'a>, Expr<'a>>>),
    Template(SpannedTemplate<'a>),
    HeredocTemplate(SpannedHeredocTemplate<'a>),
    Variable(SpannedStr<'a>),
    Conditional(Spanned<'a, Conditional<'a>>),
    FuncCall(Spanned<'a, FuncCall<'a>>),
    Traversal(Spanned<'a, Traversal<'a>>),
    UnaryOp(Spanned<'a, UnaryOp<'a>>),
    BinaryOp(Spanned<'a, BinaryOp<'a>>),
    ForExpr(Spanned<'a, ForExpr<'a>>),
}

pub enum ObjectKey<'a> {
    Identifier(SpannedStr<'a>),
    Expression(SpannedExpr<'a>),
}

pub struct Template<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct HeredocTemplate<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct Conditional<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct FuncCall<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct Traversal<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct UnaryOp<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct BinaryOp<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct ForExpr<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

// Structure

#[derive(Debug, Clone, Default)]
pub struct Body<'a> {
    pub structures: Vec<Spanned<'a, Structure<'a>>>,
}

#[derive(Debug, Clone)]
pub enum Structure<'a> {
    Attribute(Attribute<'a>),
    Block(Block<'a>),
}

#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub key: Spanned<'a, Identifier>,
    pub expr: crate::expr::Expression,
}

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub identifier: Spanned<'a, Identifier>,
    pub labels: Vec<Spanned<'a, BlockLabel>>,
    pub body: Spanned<'a, Body<'a>>,
}
