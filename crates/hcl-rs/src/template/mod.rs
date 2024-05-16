//! Types to represent the HCL template sub-language.
//!
//! When parsing an HCL document, template expressions are emitted as [`TemplateExpr`] (as the
//! `TemplateExpr` variant of the [`Expression`] enum) which contains the raw unparsed template
//! expressions.
//!
//! These template expressions can be further parsed into a [`Template`] which is composed of
//! literal strings, template interpolations and template directives.
//!
//! Refer to the [HCL syntax specification][hcl-syntax-spec] for the details.
//!
//! [hcl-syntax-spec]: https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#templates
//!
//! # Example
//!
//! Parse a `TemplateExpr` into a `Template`:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::Template;
//! use hcl::expr::{TemplateExpr, Variable};
//!
//! let expr = TemplateExpr::from("Hello ${name}!");
//! let template = Template::from_expr(&expr)?;
//!
//! let expected = Template::new()
//!     .add_literal("Hello ")
//!     .add_interpolation(Variable::new("name")?)
//!     .add_literal("!");
//!
//! assert_eq!(expected, template);
//! #   Ok(())
//! # }
//! ```
//!
//! It is also possible to use the template sub-language in a standalone way to parse template
//! strings directly:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::Identifier;
//! use hcl::expr::Variable;
//! use hcl::template::{ForDirective, StripMode, Template};
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
//!     .add_literal("\nBill of materials:\n")
//!     .add_directive(
//!         ForDirective::new(
//!             Identifier::new("item")?,
//!             Variable::new("items")?,
//!             Template::new()
//!                 .add_literal("\n- ")
//!                 .add_interpolation(Variable::new("item")?)
//!                 .add_literal("\n")
//!         )
//!         .with_for_strip(StripMode::End)
//!         .with_endfor_strip(StripMode::End)
//!     )
//!     .add_literal("\n");
//!
//! assert_eq!(expected, template);
//! #   Ok(())
//! # }
//! ```
//!
//! # Template evaluation
//!
//! The [`eval`][crate::eval] module provides evaluation capabilities for templates and
//! expressions. See the [module-level documentation][crate::eval] for examples.

mod edit;

use crate::de::FromStrVisitor;
use crate::expr::{Expression, TemplateExpr};
use crate::{format, parser, Error, Identifier, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::template::Strip;

#[doc(hidden)]
#[deprecated(since = "0.14.0", note = "use `hcl::template::Strip` instead")]
pub type StripMode = Strip;

/// The main type to represent the HCL template sub-languange.
///
/// A template behaves like an expression that always returns a string value. The different
/// elements of the template are evaluated and combined into a single string to return.
///
/// See the [`module level`][`crate::template`] documentation for usage examples.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
        Template::from_str(expr.as_str())
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

impl Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting a `Template` as string cannot fail.
        let formatted = format::to_string(self).expect("a Template failed to format unexpectedly");
        f.write_str(&formatted)
    }
}

/// An element of an HCL template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// A literal sequence of characters to include in the resulting string.
    Literal(String),
    /// An interpolation sequence that evaluates an expression (written in the expression
    /// sub-language), and converts the result to a string value.
    Interpolation(Interpolation),
    /// A `if` and `for` directive that allows for conditional template evaluation.
    Directive(Directive),
}

impl Element {
    pub(crate) fn strip(&self) -> Strip {
        match self {
            Element::Literal(_) => Strip::None,
            Element::Interpolation(interp) => interp.strip,
            Element::Directive(dir) => dir.strip(),
        }
    }
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    /// The interpolated expression.
    pub expr: Expression,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// this interpolation sequence.
    pub strip: Strip,
}

impl Interpolation {
    /// Creates a new expression `Interpolation`.
    pub fn new<T>(expr: T) -> Interpolation
    where
        T: Into<Expression>,
    {
        Interpolation {
            expr: expr.into(),
            strip: Strip::None,
        }
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after this interpolation sequence and returns the modified `Interpolation`.
    pub fn with_strip(mut self, strip: Strip) -> Interpolation {
        self.strip = strip;
        self
    }
}

impl<T> From<T> for Interpolation
where
    T: Into<Expression>,
{
    fn from(expr: T) -> Self {
        Interpolation {
            expr: expr.into(),
            strip: Strip::default(),
        }
    }
}

/// A template directive that allows for conditional template evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// Represents a template `if` directive.
    If(IfDirective),
    /// Represents a template `for` directive.
    For(ForDirective),
}

impl Directive {
    fn strip(&self) -> Strip {
        let (start, end) = match self {
            Directive::If(dir) => (dir.if_strip, dir.endif_strip),
            Directive::For(dir) => (dir.for_strip, dir.endfor_strip),
        };

        Strip::from((start.strip_start(), end.strip_end()))
    }
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
/// selection of one of two sub-templates based on the condition result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective {
    /// The condition expression.
    pub cond_expr: Expression,
    /// The template that is included in the result string if the conditional expression evaluates
    /// to `true`.
    pub true_template: Template,
    /// The template that is included in the result string if the `if` branch's conditional
    /// expression evaluates to `false`. This is `None` if there is no `else` branch in which case
    /// the result string will be empty.
    pub false_template: Option<Template>,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `if` expression.
    pub if_strip: Strip,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `else` expression. This has no effect if `false_template` is `None`.
    pub else_strip: Strip,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `endif` marker of this directive.
    pub endif_strip: Strip,
}

impl IfDirective {
    /// Creates a new `IfDirective` from a conditional expression and a template that is included
    /// in the result string if the conditional expression evaluates to `true`.
    pub fn new<T>(cond_expr: T, true_template: Template) -> IfDirective
    where
        T: Into<Expression>,
    {
        IfDirective {
            cond_expr: cond_expr.into(),
            true_template,
            false_template: None,
            if_strip: Strip::default(),
            else_strip: Strip::default(),
            endif_strip: Strip::default(),
        }
    }

    /// Adds a template for the `else` branch which is included in the result string if the
    /// condition of the `IfDirective` evaluates to `false` and returns the modified `IfDirective`.
    pub fn with_false_template<T>(mut self, else_template: T) -> IfDirective
    where
        T: Into<Template>,
    {
        self.false_template = Some(else_template.into());
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `if` expression and returns the modified `IfDirective`.
    pub fn with_if_strip(mut self, strip: Strip) -> IfDirective {
        self.if_strip = strip;
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `else` expression and returns the modified `IfDirective`.
    pub fn with_else_strip(mut self, strip: Strip) -> IfDirective {
        self.else_strip = strip;
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `endif` marker of this directive and returns the modified `IfDirective`.
    pub fn with_endif_strip(mut self, strip: Strip) -> IfDirective {
        self.endif_strip = strip;
        self
    }
}

/// The template `for` directive is the template equivalent of the for expression, producing zero
/// or more copies of its sub-template based on the elements of a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    /// Optional iterator key variable identifier.
    pub key_var: Option<Identifier>,
    /// The iterator value variable identifier.
    pub value_var: Identifier,
    /// The expression that produces the list or object of elements to iterate over.
    pub collection_expr: Expression,
    /// The template that is included in the result string for each loop iteration.
    pub template: Template,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `for` expression.
    pub for_strip: Strip,
    /// The whitespace strip mode to use on the template elements preceeding and following after
    /// the `endfor` marker of this directive.
    pub endfor_strip: Strip,
}

impl ForDirective {
    /// Creates a new `ForDirective` from the provided iterator value identifier, an expression
    /// that produces the list or object of elements to iterate over, and the template the is
    /// included in the result string for each loop iteration.
    pub fn new<T>(value: Identifier, collection_expr: T, template: Template) -> ForDirective
    where
        T: Into<Expression>,
    {
        ForDirective {
            key_var: None,
            value_var: value,
            collection_expr: collection_expr.into(),
            template,
            for_strip: Strip::default(),
            endfor_strip: Strip::default(),
        }
    }

    /// Adds the iterator key variable identifier to the `for` expression and returns the modified
    /// `ForDirective`.
    pub fn with_key_var(mut self, key_var: Identifier) -> ForDirective {
        self.key_var = Some(key_var);
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `for` expression and returns the modified `ForDirective`.
    pub fn with_for_strip(mut self, strip: Strip) -> ForDirective {
        self.for_strip = strip;
        self
    }

    /// Sets the whitespace strip mode to use on the template elements preceeding and following
    /// after the `endfor` marker of this directive and returns the modified `ForDirective`.
    pub fn with_endfor_strip(mut self, strip: Strip) -> ForDirective {
        self.endfor_strip = strip;
        self
    }
}

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FromStrVisitor::<Self>::new("a template"))
    }
}
