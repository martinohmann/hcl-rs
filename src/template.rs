//! This module implements the HCL template sub-language.
//!
//! When parsing an HCL document, template expressions are emitted as
//! [`TemplateExpr`][`crate::structure::TemplateExpr`] (as the `TemplateExpr` variant of the
//! [`Expression`][`crate::structure::Expression`] enum) which contain the raw unparsed template
//! strings.
//!
//! These template expression can be further parsed into a [`Template`] which is composed literal
//! strings, template interpolations and template directives.
//!
//! Refer to the [HCL syntax specification][hcl-syntax-spec] for the detail.
//!
//! [hcl-syntax-spec]: https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#templates
//!
//! ## Example
//!
//! Parse a `TemplateExpr` into a `Template`:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::template::Template;
//! use hcl::{Expression, Identifier, TemplateExpr};
//!
//! let expr = TemplateExpr::QuotedString(String::from("Hello ${name}!"));
//! let template = Template::from_expr(&expr)?;
//!
//! let expected = Template::new()
//!     .add_literal("Hello ")
//!     .add_interpolation(Expression::VariableExpr(Identifier::new("name")))
//!     .add_literal("!");
//!
//! assert_eq!(expected, template);
//! #   Ok(())
//! # }
//! ```
//!
//! It is also possible to use the template sub-language in a standalone way by parsing template
//! strings directly:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::{template::{ForDirective, ForExpr, StripMode, Template}};
//! use hcl::{Expression, Identifier};
//! use std::str::FromStr;
//!
//! let raw = r#"
//! Bill of materials:
//! %{ for item in items ~}
//! - ${item}
//! %{ endfor ~}
//! "#;
//!
//! let template = Template::from_str(raw)?;
//!
//! let expected = Template::new()
//!     .add_literal("Bill of materials:\n")
//!     .add_directive(
//!         ForDirective::new(
//!             ForExpr::new(
//!                 Identifier::new("item"),
//!                 Expression::VariableExpr(Identifier::new("items")),
//!                 Template::new()
//!                     .add_literal("- ")
//!                     .add_interpolation(
//!                         Expression::VariableExpr(Identifier::new("item"))
//!                     )
//!                     .add_literal("\n")
//!             )
//!             .with_strip_mode(StripMode::End)
//!         )
//!         .with_strip_mode(StripMode::End)
//!     )
//!     .add_literal("\n");
//!
//! assert_eq!(expected, template);
//! #   Ok(())
//! # }
//! ```

use crate::{parser, structure::Identifier, Error, Expression, Result, TemplateExpr};
use std::str::FromStr;

/// A template behaves like an expression that always returns a string value. The different
/// elements of the template are evaluated and combined into a single string to return.
///
/// See the [`module level`][`crate::template`] documentation for usage examples.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Template {
    elements: Vec<Element>,
}

impl Template {
    /// Creates an empty template with no elements.
    pub fn new() -> Template {
        Template {
            elements: Vec::new(),
        }
    }

    /// Expands a raw template expression to a template.
    ///
    /// ## Errors
    ///
    /// Returns an error if the parsing of raw string templates fails or if the template expression
    /// contains string literals with invalid escape sequences.
    pub fn from_expr(expr: &TemplateExpr) -> Result<Self> {
        Template::from_str(&expr.to_cow_str())
    }

    /// Returns a reference to the template elements.
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// Returns a mutable reference to the template elements.
    pub fn elements_mut(&mut self) -> &mut [Element] {
        &mut self.elements
    }
}

// Builder methods.
impl Template {
    /// Adds a template element (literal, interpolation or directive) to the template.
    pub fn add_element<T>(mut self, element: T) -> Template
    where
        T: Into<Element>,
    {
        self.elements.push(element.into());
        self
    }

    /// Adds a literal to the template.
    pub fn add_literal<T>(self, literal: T) -> Template
    where
        T: Into<String>,
    {
        self.add_element(literal.into())
    }

    /// Adds an interpolation to the template.
    pub fn add_interpolation<T>(self, interpolation: T) -> Template
    where
        T: Into<Interpolation>,
    {
        self.add_element(interpolation.into())
    }

    /// Adds a directive to the template.
    pub fn add_directive<T>(self, directive: T) -> Template
    where
        T: Into<Directive>,
    {
        self.add_element(directive.into())
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

/// An element of an HCL template.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// A literal sequence of characters to include in the resulting string.
    Literal(String),
    /// An interpolation sequence that evaluates an expression (written in the expression
    /// sub-language), and converts the result to a string value.
    Interpolation(Interpolation),
    /// A `if` and `for` directive that allows for conditional template evaluation.
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

/// An interpolation sequence evaluates an expression (written in the expression sub-language),
/// converts the result to a string value, and replaces itself with the resulting string.
#[derive(Debug, Clone, PartialEq)]
pub struct Interpolation {
    /// The interpolated expression.
    pub expr: Expression,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// this interpolation sequence.
    pub strip: StripMode,
}

impl Interpolation {
    /// Creates a new expression `Interpolation`.
    pub fn new<T>(expr: T) -> Interpolation
    where
        T: Into<Expression>,
    {
        Interpolation {
            expr: expr.into(),
            strip: StripMode::None,
        }
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after this interpolation sequence and returns the modified `Interpolation`.
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

/// A template directive that allows for conditional template evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// Represents a template `if` directive.
    If(IfDirective),
    /// Represents a template `for` directive.
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

/// The template `if` directive is the template equivalent of the conditional expression, allowing
/// selection of one of two sub-templates based on the value of a predicate expression.
#[derive(Debug, Clone, PartialEq)]
pub struct IfDirective {
    /// The `if` branch expression.
    pub if_expr: IfExpr,
    /// The optional `else` branch expression.
    pub else_expr: Option<ElseExpr>,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `endif` marker of this directive.
    pub strip: StripMode,
}

impl IfDirective {
    /// Creates a new `IfDirective` from an `if` expression.
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

    /// Adds an `else` expression and returns the modified `IfDirective`.
    pub fn with_else_expr<T>(mut self, else_expr: T) -> IfDirective
    where
        T: Into<ElseExpr>,
    {
        self.else_expr = Some(else_expr.into());
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `endif` marker of this directive and returns the modified `IfDirective`.
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

/// The `if` branch of an `if` directive.
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    /// The conditional expression.
    pub expr: Expression,
    /// The template that is included in the result string if the conditional expression evaluates
    /// to `true`.
    pub template: Template,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `if` expression.
    pub strip: StripMode,
}

impl IfExpr {
    /// Creates a new `IfExpr` from an expression and a template that is included in the result
    /// string if the conditional expression evaluates to `true`.
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

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `if` expression and returns the modified `IfExpr`.
    pub fn with_strip_mode(mut self, strip: StripMode) -> IfExpr {
        self.strip = strip;
        self
    }
}

/// The `else` branch expression of an `if` directive.
#[derive(Debug, Clone, PartialEq)]
pub struct ElseExpr {
    /// The template that is included in the result string if the `if` branch's conditional
    /// expression evaluates to `false`.
    pub template: Template,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `else` expression.
    pub strip: StripMode,
}

impl ElseExpr {
    /// Creates a new `ElseExpr` from a template that is included in the result string if the `if`
    /// branch's conditional expression evaluates to `false`.
    pub fn new(template: Template) -> ElseExpr {
        ElseExpr {
            template,
            strip: StripMode::default(),
        }
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `else` expression and returns the modified `ElseExpr`.
    pub fn with_strip_mode(mut self, strip: StripMode) -> ElseExpr {
        self.strip = strip;
        self
    }
}

/// The template `for` directive is the template equivalent of the for expression, producing zero
/// or more copies of its sub-template based on the elements of a collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ForDirective {
    /// The loop expression.
    pub for_expr: ForExpr,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `endfor` marker of this directive.
    pub strip: StripMode,
}

impl ForDirective {
    /// Creates a new `ForDirective` from a `for` expression.
    pub fn new<T>(for_expr: T) -> ForDirective
    where
        T: Into<ForExpr>,
    {
        ForDirective {
            for_expr: for_expr.into(),
            strip: StripMode::default(),
        }
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `endfor` marker of this directive and returns the modified `ForDirective`.
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

/// The `for` expression header of a `for` directive.
#[derive(Debug, Clone, PartialEq)]
pub struct ForExpr {
    /// Optional iterator key variable identifier.
    pub key: Option<Identifier>,
    /// The iterator value variable identifier.
    pub value: Identifier,
    /// The expression that produces the list or object of elements to iterate over.
    pub expr: Expression,
    /// The template that is included in the result string for each loop iteration.
    pub template: Template,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `for` expression.
    pub strip: StripMode,
}

impl ForExpr {
    /// Creates a new `ForExpr` from the provided iterator value identifier, an expression that
    /// produces the list or object of elements to iterate over, and the template the is included
    /// in the result string for each loop iteration.
    pub fn new<T>(value: Identifier, expr: T, template: Template) -> ForExpr
    where
        T: Into<Expression>,
    {
        ForExpr {
            key: None,
            value,
            expr: expr.into(),
            template,
            strip: StripMode::default(),
        }
    }

    /// Adds the iterator key variable identifier to the `for` expression and returns the modified
    /// `ForExpr`.
    pub fn with_key(mut self, key: Identifier) -> ForExpr {
        self.key = Some(key);
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `for` expression and returns the modified `ForExpr`.
    pub fn with_strip_mode(mut self, strip: StripMode) -> ForExpr {
        self.strip = strip;
        self
    }
}

/// Controls the whitespace strip behaviour on adjacent string literals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripMode {
    /// Don't strip adjacent spaces.
    None,
    /// Strip any adjacent spaces from the immediately preceeding string literal, if there is
    /// one.
    Start,
    /// Strip any adjacent spaces from the immediately following string literal, if there is one.
    End,
    /// Strip any adjacent spaces from the immediately preceeding and following string literals,
    /// if there are any.
    Both,
}

impl Default for StripMode {
    fn default() -> StripMode {
        StripMode::None
    }
}
