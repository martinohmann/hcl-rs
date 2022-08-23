use crate::{parser, structure::Identifier, Error, Expression, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub elements: Vec<Element>,
}

impl Template {
    pub fn new() -> Template {
        Template {
            elements: Vec::new(),
        }
    }

    pub fn add_element<T>(mut self, element: T) -> Template
    where
        T: Into<Element>,
    {
        self.elements.push(element.into());
        self
    }

    pub fn add_literal<T>(self, literal: T) -> Template
    where
        T: Into<String>,
    {
        self.add_element(literal.into())
    }

    pub fn add_interpolation<T>(self, interpolation: T) -> Template
    where
        T: Into<Interpolation>,
    {
        self.add_element(interpolation.into())
    }

    pub fn add_directive<T>(self, directive: T) -> Template
    where
        T: Into<Directive>,
    {
        self.add_element(directive.into())
    }

    pub fn add_if_directive<T>(self, directive: T) -> Template
    where
        T: Into<IfDirective>,
    {
        self.add_directive(directive.into())
    }

    pub fn add_for_directive<T>(self, directive: T) -> Template
    where
        T: Into<ForDirective>,
    {
        self.add_directive(directive.into())
    }
}

impl FromStr for Template {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_template(s)
    }
}

impl<T> FromIterator<T> for Template
where
    T: Into<Element>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Template {
            elements: iter.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Literal(String),
    Interpolation(Interpolation),
    Directive(Directive),
}

impl From<&str> for Element {
    fn from(literal: &str) -> Self {
        Element::Literal(literal.to_owned())
    }
}

impl From<String> for Element {
    fn from(literal: String) -> Self {
        Element::Literal(literal)
    }
}

impl From<Interpolation> for Element {
    fn from(interpolation: Interpolation) -> Self {
        Element::Interpolation(interpolation)
    }
}

impl From<Directive> for Element {
    fn from(directive: Directive) -> Self {
        Element::Directive(directive)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolation {
    pub expr: Expression,
    pub strip: StripMode,
}

impl Interpolation {
    pub fn new<T>(expr: T) -> Interpolation
    where
        T: Into<Expression>,
    {
        Interpolation {
            expr: expr.into(),
            strip: StripMode::None,
        }
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> Interpolation {
        self.strip = strip;
        self
    }
}

impl From<Expression> for Interpolation {
    fn from(expr: Expression) -> Self {
        Interpolation {
            expr,
            strip: StripMode::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    If(IfDirective),
    For(ForDirective),
}

impl From<IfDirective> for Directive {
    fn from(directive: IfDirective) -> Self {
        Directive::If(directive)
    }
}

impl From<ForDirective> for Directive {
    fn from(directive: ForDirective) -> Self {
        Directive::For(directive)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfDirective {
    pub if_expr: IfExpr,
    pub else_expr: Option<ElseExpr>,
    pub strip: StripMode,
}

impl IfDirective {
    pub fn new<T>(if_expr: T) -> IfDirective
    where
        T: Into<IfExpr>,
    {
        IfDirective {
            if_expr: if_expr.into(),
            else_expr: None,
            strip: StripMode::default(),
        }
    }

    pub fn with_else_expr<T>(mut self, else_expr: T) -> IfDirective
    where
        T: Into<ElseExpr>,
    {
        self.else_expr = Some(else_expr.into());
        self
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> IfDirective {
        self.strip = strip;
        self
    }
}

impl From<IfExpr> for IfDirective {
    fn from(if_expr: IfExpr) -> Self {
        IfDirective::new(if_expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub expr: Expression,
    pub template: Template,
    pub strip: StripMode,
}

impl IfExpr {
    pub fn new<T>(expr: T, template: Template) -> IfExpr
    where
        T: Into<Expression>,
    {
        IfExpr {
            expr: expr.into(),
            template,
            strip: StripMode::default(),
        }
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> IfExpr {
        self.strip = strip;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseExpr {
    pub template: Template,
    pub strip: StripMode,
}

impl ElseExpr {
    pub fn new(template: Template) -> ElseExpr {
        ElseExpr {
            template,
            strip: StripMode::default(),
        }
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> ElseExpr {
        self.strip = strip;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForDirective {
    pub for_expr: ForExpr,
    pub strip: StripMode,
}

impl ForDirective {
    pub fn new<T>(for_expr: T) -> ForDirective
    where
        T: Into<ForExpr>,
    {
        ForDirective {
            for_expr: for_expr.into(),
            strip: StripMode::default(),
        }
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> ForDirective {
        self.strip = strip;
        self
    }
}

impl From<ForExpr> for ForDirective {
    fn from(for_expr: ForExpr) -> Self {
        ForDirective::new(for_expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForExpr {
    pub key: Identifier,
    pub value: Option<Identifier>,
    pub expr: Expression,
    pub template: Template,
    pub strip: StripMode,
}

impl ForExpr {
    pub fn new<K, T>(key: K, expr: T, template: Template) -> ForExpr
    where
        K: Into<Identifier>,
        T: Into<Expression>,
    {
        ForExpr {
            key: key.into(),
            value: None,
            expr: expr.into(),
            template,
            strip: StripMode::default(),
        }
    }

    pub fn with_value<T>(mut self, value: T) -> ForExpr
    where
        T: Into<Identifier>,
    {
        self.value = Some(value.into());
        self
    }

    pub fn with_strip_mode(mut self, strip: StripMode) -> ForExpr {
        self.strip = strip;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripMode {
    None,
    Start,
    End,
    Both,
}

impl Default for StripMode {
    fn default() -> StripMode {
        StripMode::None
    }
}
