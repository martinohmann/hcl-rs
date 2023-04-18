//! Types to represent the HCL template sub-language.

#![allow(missing_docs)]

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

#[derive(Debug, Clone, Eq)]
pub struct HeredocTemplate {
    pub delimiter: Ident,
    pub template: Template,

    indent: Option<usize>,
    trailing: RawString,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl HeredocTemplate {
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

    pub fn indent(&self) -> Option<usize> {
        self.indent
    }

    pub fn set_indent(&mut self, indent: usize) {
        self.indent = Some(indent);
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Literal(Spanned<String>),
    Interpolation(Interpolation),
    Directive(Directive),
}

impl Element {
    pub(crate) fn despan(&mut self, input: &str) {
        match self {
            Element::Literal(_) => {}
            Element::Interpolation(interp) => interp.despan(input),
            Element::Directive(dir) => dir.despan(input),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Interpolation {
    pub expr: Expression,
    pub strip: Strip,

    span: Option<Range<usize>>,
}

impl Interpolation {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    If(IfDirective),
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

#[derive(Debug, Clone, Eq)]
pub struct IfDirective {
    pub if_expr: IfTemplateExpr,
    pub else_expr: Option<ElseTemplateExpr>,
    pub endif_expr: EndifTemplateExpr,

    span: Option<Range<usize>>,
}

impl IfDirective {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfTemplateExpr {
    pub cond_expr: Expression,
    pub template: Template,
    pub strip: Strip,

    preamble: RawString,
}

impl IfTemplateExpr {
    pub fn new(cond_expr: Expression, template: Template) -> IfTemplateExpr {
        IfTemplateExpr {
            preamble: RawString::default(),
            cond_expr,
            template,
            strip: Strip::default(),
        }
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.cond_expr.despan(input);
        self.template.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElseTemplateExpr {
    pub template: Template,
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl ElseTemplateExpr {
    pub fn new(template: Template) -> ElseTemplateExpr {
        ElseTemplateExpr {
            preamble: RawString::default(),
            trailing: RawString::default(),
            template,
            strip: Strip::default(),
        }
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.template.despan(input);
        self.trailing.despan(input);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndifTemplateExpr {
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl EndifTemplateExpr {
    pub fn new() -> EndifTemplateExpr {
        EndifTemplateExpr::default()
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.preamble.despan(input);
        self.trailing.despan(input);
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ForDirective {
    pub for_expr: ForTemplateExpr,
    pub endfor_expr: EndforTemplateExpr,

    span: Option<Range<usize>>,
}

impl ForDirective {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForTemplateExpr {
    pub key_var: Option<Decorated<Ident>>,
    pub value_var: Decorated<Ident>,
    pub collection_expr: Expression,
    pub template: Template,
    pub strip: Strip,

    preamble: RawString,
}

impl ForTemplateExpr {
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

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EndforTemplateExpr {
    pub strip: Strip,

    preamble: RawString,
    trailing: RawString,
}

impl EndforTemplateExpr {
    pub fn new() -> EndforTemplateExpr {
        EndforTemplateExpr::default()
    }

    pub fn preamble(&self) -> &RawString {
        &self.preamble
    }

    pub fn set_preamble(&mut self, preamble: impl Into<RawString>) {
        self.preamble = preamble.into();
    }

    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

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
