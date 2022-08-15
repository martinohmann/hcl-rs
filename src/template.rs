use crate::{parser, structure::Identifier, Error, Expression, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub elements: Vec<Element>,
}

impl FromStr for Template {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_template(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Literal(String),
    Interpolation(Interpolation),
    Directive(Directive),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolation {
    pub expr: Expression,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    If(If),
    For(For),
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub if_expr: IfExpr,
    pub else_expr: Option<ElseExpr>,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub expr: Expression,
    pub template: Template,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseExpr {
    pub template: Template,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub for_expr: ForExpr,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForExpr {
    pub key: Identifier,
    pub value: Option<Identifier>,
    pub expr: Expression,
    pub template: Template,
    pub strip: StripMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StripMode {
    None,
    Start,
    End,
    Both,
}
