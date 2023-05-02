//! Types to represent the HCL template sub-language.

use crate::encode::{Encode, EncodeState};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span, Spanned};
use crate::util::{dedent_by, min_leading_whitespace};
use crate::{parser, Ident, RawString};
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::template::Strip;

/// An owning iterator over the elements of a `Template`.
///
/// Values of this type are created by the [`into_iter`] method on [`Template`] (provided by the
/// [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
/// [`IntoIterator`]: core::iter::IntoIterator
pub type IntoIter = Box<dyn Iterator<Item = Element>>;

/// An iterator over the elements of a `Template`.
///
/// Values of this type are created by the [`iter`] method on [`Template`]. See its documentation
/// for more.
///
/// [`iter`]: Template::iter
pub type Iter<'a> = Box<dyn Iterator<Item = &'a Element> + 'a>;

/// A mutable iterator over the elements of a `Template`.
///
/// Values of this type are created by the [`iter_mut`] method on [`Template`]. See its
/// documentation for more.
///
/// [`iter_mut`]: Template::iter_mut
pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Element> + 'a>;

/// A type representing the HCL template sub-languange in the context of a quoted string literal.
///
/// A template behaves like an expression that always returns a string value. The different
/// elements of the template are evaluated and combined into a single string to return.
#[derive(Debug, Clone, Eq, Default)]
pub struct StringTemplate {
    elements: Vec<Element>,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl StringTemplate {
    /// Constructs a new, empty `StringTemplate`.
    #[inline]
    pub fn new() -> Self {
        StringTemplate::default()
    }

    /// Constructs a new, empty `StringTemplate` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        StringTemplate {
            elements: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Returns `true` if the template contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the number of elements in the template, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Clears the template, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Returns a reference to the element at the given index, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }

    /// Returns a mutable reference to the element at the given index, or `None` if the index is
    /// out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Element> {
        self.elements.get_mut(index)
    }

    /// Inserts an element at position `index` within the template, shifting all elements after it
    /// to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, element: impl Into<Element>) {
        self.elements.insert(index, element.into());
    }

    /// Appends an element to the back of the template.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, element: impl Into<Element>) {
        self.elements.push(element.into());
    }

    /// Removes the last element from the template and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Element> {
        self.elements.pop()
    }

    /// Removes and returns the element at position `index` within the template, shifting all
    /// elements after it to the left.
    ///
    /// Like `Vec::remove`, the element is removed by shifting all of the elements that follow it,
    /// preserving their relative order. **This perturbs the index of all of those elements!**
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Element {
        self.elements.remove(index)
    }

    /// An iterator visiting all template elements in insertion order. The iterator element type
    /// is `&'a Element`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.elements.iter())
    }

    /// An iterator visiting all template elements in insertion order, with mutable references to
    /// the values. The iterator element type is `&'a mut Element`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.elements.iter_mut())
    }

    /// If the template consists of a single `Element`, returns a reference to it, otherwise
    /// `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::template::{Element, StringTemplate};
    ///
    /// let mut template = StringTemplate::new();
    ///
    /// template.push("one");
    ///
    /// assert_eq!(template.as_single_element(), Some(&Element::from("one")));
    ///
    /// template.push("two");
    ///
    /// assert_eq!(template.as_single_element(), None);
    /// ```
    pub fn as_single_element(&self) -> Option<&Element> {
        match self.len() {
            1 => self.get(0),
            _ => None,
        }
    }

    /// If the template consists of a single `Element`, returns a mutable reference to it,
    /// otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::template::{Element, StringTemplate};
    ///
    /// let mut template = StringTemplate::new();
    ///
    /// template.push("one");
    ///
    /// if let Some(element) = template.as_single_element_mut() {
    ///     *element = Element::from("two");
    /// }
    ///
    /// template.push("three");
    ///
    /// assert_eq!(template.as_single_element(), None);
    /// assert_eq!(template, StringTemplate::from_iter(["two", "three"]));
    /// ```
    pub fn as_single_element_mut(&mut self) -> Option<&mut Element> {
        match self.len() {
            1 => self.get_mut(0),
            _ => None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        for element in &mut self.elements {
            element.despan(input);
        }
    }
}

impl From<Vec<Element>> for StringTemplate {
    fn from(elements: Vec<Element>) -> Self {
        StringTemplate {
            elements,
            decor: Decor::default(),
            span: None,
        }
    }
}

impl PartialEq for StringTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T> Extend<T> for StringTemplate
where
    T: Into<Element>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.elements.reserve(reserve);
        iter.for_each(|v| self.push(v));
    }
}

impl<T> FromIterator<T> for StringTemplate
where
    T: Into<Element>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut template = StringTemplate::with_capacity(lower);
        template.extend(iter);
        template
    }
}

impl IntoIterator for StringTemplate {
    type Item = Element;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.elements.into_iter())
    }
}

impl<'a> IntoIterator for &'a StringTemplate {
    type Item = &'a Element;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut StringTemplate {
    type Item = &'a mut Element;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A heredoc template is introduced by a `<<` sequence and defines a template via a multi-line
/// sequence terminated by a user-chosen delimiter.
#[derive(Debug, Clone, Eq)]
pub struct HeredocTemplate {
    /// The delimiter identifier that denotes the heredoc start and end.
    pub delimiter: Ident,
    /// The raw template contained in the heredoc.
    pub template: Template,

    indent: Option<usize>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl HeredocTemplate {
    /// Creates a new `HeredocTemplate` for a delimiter and a template.
    pub fn new(delimiter: Ident, template: Template) -> HeredocTemplate {
        HeredocTemplate {
            delimiter,
            template,
            indent: None,
            trailing: RawString::default(),
            decor: Decor::default(),
            span: None,
        }
    }

    /// Return the heredoc's indent, if there is any.
    pub fn indent(&self) -> Option<usize> {
        self.indent
    }

    /// Set the heredoc's indent.
    pub fn set_indent(&mut self, indent: usize) {
        self.indent = Some(indent);
    }

    /// Return a reference to the raw trailing decor before the heredoc's closing delimiter.
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the heredoc's closing delimiter.
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    /// Dedent the heredoc.
    pub fn dedent(&mut self) {
        let mut indent: Option<usize> = None;
        let mut skip_first = false;

        for element in self.template.iter() {
            if let Element::Literal(literal) = element {
                let leading_ws = min_leading_whitespace(literal, skip_first);
                indent = Some(indent.map_or(leading_ws, |indent| indent.min(leading_ws)));
                skip_first = !literal.ends_with('\n');
            } else {
                skip_first = true;
            }
        }

        if let Some(indent) = indent {
            skip_first = false;

            for element in self.template.iter_mut() {
                if let Element::Literal(literal) = element {
                    let dedented = dedent_by(literal, indent, skip_first);
                    *literal.as_mut() = dedented.into();
                    skip_first = !literal.ends_with('\n');
                } else {
                    skip_first = true;
                }
            }

            self.set_indent(indent);
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.decor.despan(input);
        self.template.despan(input);
        self.trailing.despan(input);
    }
}

impl PartialEq for HeredocTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.delimiter == other.delimiter
            && self.template == other.template
            && self.indent == other.indent
            && self.trailing == other.trailing
    }
}

/// The main type to represent the HCL template sub-languange.
///
/// A template behaves like an expression that always returns a string value. The different
/// elements of the template are evaluated and combined into a single string to return.
#[derive(Debug, Clone, Eq, Default)]
pub struct Template {
    elements: Vec<Element>,
    span: Option<Range<usize>>,
}

impl Template {
    /// Constructs a new, empty `Template`.
    #[inline]
    pub fn new() -> Self {
        Template::default()
    }

    /// Constructs a new, empty `Template` with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Template {
            elements: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Returns `true` if the template contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns the number of elements in the template, also referred to as its 'length'.
    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Clears the template, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Returns a reference to the element at the given index, or `None` if the index is out of
    /// bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }

    /// Returns a mutable reference to the element at the given index, or `None` if the index is
    /// out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Element> {
        self.elements.get_mut(index)
    }

    /// Inserts an element at position `index` within the template, shifting all elements after it
    /// to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    #[inline]
    pub fn insert(&mut self, index: usize, element: impl Into<Element>) {
        self.elements.insert(index, element.into());
    }

    /// Appends an element to the back of the template.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    #[inline]
    pub fn push(&mut self, element: impl Into<Element>) {
        self.elements.push(element.into());
    }

    /// Removes the last element from the template and returns it, or [`None`] if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Element> {
        self.elements.pop()
    }

    /// Removes and returns the element at position `index` within the template, shifting all
    /// elements after it to the left.
    ///
    /// Like `Vec::remove`, the element is removed by shifting all of the elements that follow it,
    /// preserving their relative order. **This perturbs the index of all of those elements!**
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> Element {
        self.elements.remove(index)
    }

    /// An iterator visiting all template elements in insertion order. The iterator element type
    /// is `&'a Element`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.elements.iter())
    }

    /// An iterator visiting all template elements in insertion order, with mutable references to
    /// the values. The iterator element type is `&'a mut Element`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.elements.iter_mut())
    }

    /// If the template consists of a single `Element`, returns a reference to it, otherwise
    /// `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::template::{Element, Template};
    ///
    /// let mut template = Template::new();
    ///
    /// template.push("one");
    ///
    /// assert_eq!(template.as_single_element(), Some(&Element::from("one")));
    ///
    /// template.push("two");
    ///
    /// assert_eq!(template.as_single_element(), None);
    /// ```
    pub fn as_single_element(&self) -> Option<&Element> {
        match self.len() {
            1 => self.get(0),
            _ => None,
        }
    }

    /// If the template consists of a single `Element`, returns a mutable reference to it,
    /// otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use hcl_edit::template::{Element, Template};
    ///
    /// let mut template = Template::new();
    ///
    /// template.push("one");
    ///
    /// if let Some(element) = template.as_single_element_mut() {
    ///     *element = Element::from("two");
    /// }
    ///
    /// template.push("three");
    ///
    /// assert_eq!(template.as_single_element(), None);
    /// assert_eq!(template, Template::from_iter(["two", "three"]));
    /// ```
    pub fn as_single_element_mut(&mut self) -> Option<&mut Element> {
        match self.len() {
            1 => self.get_mut(0),
            _ => None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        for element in &mut self.elements {
            element.despan(input);
        }
    }
}

impl PartialEq for Template {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

impl FromStr for Template {
    type Err = parser::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse_template(s)
    }
}

impl From<Vec<Element>> for Template {
    fn from(elements: Vec<Element>) -> Self {
        Template {
            elements,
            ..Default::default()
        }
    }
}

impl<T> Extend<T> for Template
where
    T: Into<Element>,
{
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.elements.reserve(reserve);
        iter.for_each(|v| self.push(v));
    }
}

impl<T> FromIterator<T> for Template
where
    T: Into<Element>,
{
    fn from_iter<I>(iterable: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iterable.into_iter();
        let lower = iter.size_hint().0;
        let mut template = Template::with_capacity(lower);
        template.extend(iter);
        template
    }
}

impl IntoIterator for Template {
    type Item = Element;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.elements.into_iter())
    }
}

impl<'a> IntoIterator for &'a Template {
    type Item = &'a Element;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Template {
    type Item = &'a mut Element;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An element of an HCL template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// A literal sequence of characters to include in the resulting string.
    Literal(Spanned<String>),
    /// An interpolation sequence that evaluates an expression (written in the expression
    /// sub-language), and converts the result to a string value.
    Interpolation(Interpolation),
    /// An `if` or `for` directive that allows for conditional template evaluation.
    Directive(Directive),
}

impl Element {
    /// Returns `true` if the element represents a literal string.
    pub fn is_literal(&self) -> bool {
        self.as_literal().is_some()
    }

    /// If the `Element` is a literal string, returns a reference to it, otherwise `None`.
    pub fn as_literal(&self) -> Option<&Spanned<String>> {
        match self {
            Element::Literal(value) => Some(value),
            Element::Interpolation(_) | Element::Directive(_) => None,
        }
    }

    /// Returns `true` if the element represents an interpolation.
    pub fn is_interpolation(&self) -> bool {
        self.as_interpolation().is_some()
    }

    /// If the `Element` is an interpolation, returns a reference to it, otherwise `None`.
    pub fn as_interpolation(&self) -> Option<&Interpolation> {
        match self {
            Element::Interpolation(value) => Some(value),
            Element::Literal(_) | Element::Directive(_) => None,
        }
    }

    /// Returns `true` if the element represents a directive.
    pub fn is_directive(&self) -> bool {
        self.as_directive().is_some()
    }

    /// If the `Element` is a directive, returns a reference to it, otherwise `None`.
    pub fn as_directive(&self) -> Option<&Directive> {
        match self {
            Element::Directive(value) => Some(value),
            Element::Literal(_) | Element::Interpolation(_) => None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Element::Literal(_) => {}
            Element::Interpolation(interp) => interp.despan(input),
            Element::Directive(dir) => dir.despan(input),
        }
    }
}

impl From<&str> for Element {
    fn from(value: &str) -> Self {
        Element::from(value.to_string())
    }
}

impl From<String> for Element {
    fn from(value: String) -> Self {
        Element::from(Spanned::new(value))
    }
}

impl From<Spanned<String>> for Element {
    fn from(value: Spanned<String>) -> Self {
        Element::Literal(value)
    }
}

impl From<Interpolation> for Element {
    fn from(value: Interpolation) -> Self {
        Element::Interpolation(value)
    }
}

impl From<Directive> for Element {
    fn from(value: Directive) -> Self {
        Element::Directive(value)
    }
}

/// An interpolation sequence evaluates an expression (written in the expression sub-language),
/// converts the result to a string value, and replaces itself with the resulting string.
#[derive(Debug, Clone, Eq)]
pub struct Interpolation {
    /// The interpolated expression.
    pub expr: Expression,
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after this interpolation sequence.
    pub strip: Strip,

    span: Option<Range<usize>>,
}

impl Interpolation {
    /// Creates a new `Interpolation` from an expression.
    pub fn new(expr: Expression) -> Interpolation {
        Interpolation {
            expr,
            strip: Strip::default(),
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.expr.despan(input);
    }
}

impl PartialEq for Interpolation {
    fn eq(&self, other: &Self) -> bool {
        self.expr == other.expr && self.strip == other.strip
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
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Directive::If(dir) => dir.despan(input),
            Directive::For(dir) => dir.despan(input),
        }
    }
}

impl From<IfDirective> for Directive {
    fn from(value: IfDirective) -> Self {
        Directive::If(value)
    }
}

impl From<ForDirective> for Directive {
    fn from(value: ForDirective) -> Self {
        Directive::For(value)
    }
}

/// The template `if` directive is the template equivalent of the conditional expression, allowing
/// selection of one of two sub-templates based on the condition result.
#[derive(Debug, Clone, Eq)]
pub struct IfDirective {
    /// The `if` sub-expression within the directive.
    pub if_expr: IfTemplateExpr,
    /// The `else` sub-expression within the directive. This is `None` if there is no `else`
    /// branch in which case the result string will be empty.
    pub else_expr: Option<ElseTemplateExpr>,
    /// The `endif` sub-expression within the directive.
    pub endif_expr: EndifTemplateExpr,

    span: Option<Range<usize>>,
}

impl IfDirective {
    /// Creates a new `IfDirective` from the parts for the `if`, `else` and `endif`
    /// sub-expressions.
    pub fn new(
        if_expr: IfTemplateExpr,
        else_expr: Option<ElseTemplateExpr>,
        endif_expr: EndifTemplateExpr,
    ) -> IfDirective {
        IfDirective {
            if_expr,
            else_expr,
            endif_expr,
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.if_expr.despan(input);

        if let Some(else_expr) = &mut self.else_expr {
            else_expr.despan(input);
        }

        self.endif_expr.despan(input);
    }
}

impl PartialEq for IfDirective {
    fn eq(&self, other: &Self) -> bool {
        self.if_expr == other.if_expr
            && self.else_expr == other.else_expr
            && self.endif_expr == other.endif_expr
    }
}

/// A type representing the `%{ if cond_expr }` sub-expression and the template that follows after
/// it within an [`IfDirective`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfTemplateExpr {
    /// The condition expression.
    pub cond_expr: Expression,
    /// The template that is included in the result string if the conditional expression evaluates
    /// to `true`.
    pub template: Template,
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after the `if` expression.
    pub strip: Strip,

    preamble: RawString,
}

impl IfTemplateExpr {
    /// Creates a new `IfTemplateExpr` for a condition expression and a template.
    pub fn new(cond_expr: Expression, template: Template) -> IfTemplateExpr {
        IfTemplateExpr {
            preamble: RawString::default(),
            cond_expr,
            template,
            strip: Strip::default(),
        }
    }

    /// Return a reference to the raw leading decor after the `if`'s opening `{`.
    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    /// Set the raw leading decor after the `if`'s opening `{`.
    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.cond_expr.despan(input);
        self.template.despan(input);
    }
}

/// A type representing the `%{ else }` sub-expression and the template that follows after it
/// within an [`IfDirective`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseTemplateExpr {
    /// The template that is included in the result string if the `if` branch's conditional
    /// expression evaluates to `false`.
    pub template: Template,
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after the `else` expression.
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl ElseTemplateExpr {
    /// Creates a new `ElseTemplateExpr` for a template.
    pub fn new(template: Template) -> ElseTemplateExpr {
        ElseTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            template,
            strip: Strip::default(),
        }
    }

    /// Return a reference to the raw leading decor after the `else`'s opening `{`.
    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    /// Set the raw leading decor after the `else`'s opening `{`.
    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    /// Return a reference to the raw trailing decor before the `else`'s closing `}`.
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the `else`'s closing `}`.
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.template.despan(input);
        self.trailing.despan(input);
    }
}

/// A type representing the `%{ endif }` sub-expression within an [`IfDirective`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndifTemplateExpr {
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after the `endif` marker.
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl EndifTemplateExpr {
    /// Creates a new `EndifTemplateExpr`.
    pub fn new() -> EndifTemplateExpr {
        EndifTemplateExpr::default()
    }

    /// Return a reference to the raw leading decor after the `endif`'s opening `{`.
    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    /// Set the raw leading decor after the `endif`'s opening `{`.
    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    /// Return a reference to the raw trailing decor before the `endif`'s closing `}`.
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the `endif`'s closing `}`.
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.trailing.despan(input);
    }
}

/// The template `for` directive is the template equivalent of the for expression, producing zero
/// or more copies of its sub-template based on the elements of a collection.
#[derive(Debug, Clone, Eq)]
pub struct ForDirective {
    /// The `for` sub-expression within the directive.
    pub for_expr: ForTemplateExpr,
    /// The `endfor` sub-expression within the directive.
    pub endfor_expr: EndforTemplateExpr,

    span: Option<Range<usize>>,
}

impl ForDirective {
    /// Creates a new `ForDirective` from the parts for the `for` and `endfor` sub-expressions.
    pub fn new(for_expr: ForTemplateExpr, endfor_expr: EndforTemplateExpr) -> ForDirective {
        ForDirective {
            for_expr,
            endfor_expr,
            span: None,
        }
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.for_expr.despan(input);
        self.endfor_expr.despan(input);
    }
}

impl PartialEq for ForDirective {
    fn eq(&self, other: &Self) -> bool {
        self.for_expr == other.for_expr && self.endfor_expr == other.endfor_expr
    }
}

/// A type representing the `%{ for key_var, value_var in collection_expr }` sub-expression and
/// the template that follows after it within a [`ForDirective`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForTemplateExpr {
    /// Optional iterator key variable identifier.
    pub key_var: Option<Decorated<Ident>>,
    /// The iterator value variable identifier.
    pub value_var: Decorated<Ident>,
    /// The expression that produces the list or object of elements to iterate over.
    pub collection_expr: Expression,
    /// The template that is included in the result string for each loop iteration.
    pub template: Template,
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after the `for` expression.
    pub strip: Strip,

    preamble: RawString,
}

impl ForTemplateExpr {
    /// Creates a new `ForTemplateExpr` from an optional key variable, value variable, collection
    /// expression and template.
    pub fn new(
        key_var: Option<Decorated<Ident>>,
        value_var: Decorated<Ident>,
        collection_expr: Expression,
        template: Template,
    ) -> ForTemplateExpr {
        ForTemplateExpr {
            preamble: RawString::default(),
            key_var,
            value_var,
            collection_expr,
            template,
            strip: Strip::default(),
        }
    }

    /// Return a reference to the raw leading decor after the `for`'s opening `{`.
    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    /// Set the raw leading decor after the `for`'s opening `{`.
    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);

        if let Some(key_var) = &mut self.key_var {
            key_var.decor_mut().despan(input);
        }

        self.value_var.decor_mut().despan(input);
        self.collection_expr.despan(input);
        self.template.despan(input);
    }
}

/// A type representing the `%{ endfor }` sub-expression within a [`ForDirective`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndforTemplateExpr {
    /// The whitespace strip behaviour to use on the template elements preceeding and following
    /// after the `endfor` marker.
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl EndforTemplateExpr {
    /// Creates a new `EndforTemplateExpr`.
    pub fn new() -> EndforTemplateExpr {
        EndforTemplateExpr::default()
    }

    /// Return a reference to the raw leading decor after the `endfor`'s opening `{`.
    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    /// Set the raw leading decor after the `endfor`'s opening `{`.
    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    /// Return a reference to the raw trailing decor before the `endfor`'s closing `}`.
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// Set the raw trailing decor before the `endfor`'s closing `}`.
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.trailing.despan(input);
    }
}

decorate_impl! { StringTemplate, HeredocTemplate }

span_impl! {
    StringTemplate, HeredocTemplate, Template,
    Interpolation, IfDirective, ForDirective
}

forward_span_impl! {
    Element => { Literal, Interpolation, Directive },
    Directive => { If, For }
}
