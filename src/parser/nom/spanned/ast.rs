use crate::Number;
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

pub struct Body<'a> {
    pub structures: Spanned<'a, Vec<Structure<'a>>>,
}

pub enum Structure<'a> {
    Attribute(Spanned<'a, Attribute<'a>>),
    Block(Spanned<'a, Block<'a>>),
}

pub struct Attribute<'a> {
    pub key: SpannedStr<'a>,
    pub expr: SpannedExpr<'a>,
}

pub struct Block<'a> {
    pub ident: SpannedStr<'a>,
    pub labels: Spanned<'a, Vec<BlockLabel<'a>>>,
    pub body: Spanned<'a, Body<'a>>,
}

pub enum BlockLabel<'a> {
    Identifier(SpannedStr<'a>),
    String(SpannedStr<'a>),
}
