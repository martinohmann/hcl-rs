use crate::encode::{Encode, EncodeState};
use crate::expr::Expression;
use crate::repr::{Decor, Decorate, Decorated, SetSpan, Span, Spanned};
use crate::util::{dedent_by, min_leading_whitespace};
use crate::{Ident, InternalString, RawString};
use std::fmt;
use std::ops::Range;

// Re-exported for convenience.
#[doc(inline)]
pub use hcl_primitives::template::Strip;

pub type Iter<'a> = Box<dyn Iterator<Item = &'a Element> + 'a>;

pub type IterMut<'a> = Box<dyn Iterator<Item = &'a mut Element> + 'a>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringTemplate {
    elements: Vec<Element>,
    decor: Decor,
    span: Option<Range<usize>>,
}

impl StringTemplate {
    pub fn new(elements: Vec<Element>) -> StringTemplate {
        StringTemplate {
            elements,
            decor: Decor::default(),
            span: None,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.elements.iter())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocTemplate {
    delimiter: Ident,
    template: Template,
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

    pub fn delimiter(&self) -> &Ident {
        &self.delimiter
    }

    pub fn template(&self) -> &Template {
        &self.template
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    elements: Vec<Element>,
    span: Option<Range<usize>>,
}

impl Template {
    pub fn new(elements: Vec<Element>) -> Template {
        Template {
            elements,
            span: None,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Box::new(self.elements.iter())
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        Box::new(self.elements.iter_mut())
    }

    pub(crate) fn despan(&mut self, input: &str) {
        for element in &mut self.elements {
            element.despan(input);
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut state = EncodeState::new(f);
        self.encode(&mut state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    Literal(Spanned<InternalString>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpolation {
    expr: Expression,
    strip: Strip,
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

    pub fn expr(&self) -> &Expression {
        &self.expr
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.expr.despan(input);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfDirective {
    if_expr: IfTemplateExpr,
    else_expr: Option<ElseTemplateExpr>,
    endif_expr: EndifTemplateExpr,
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

    pub fn if_expr(&self) -> &IfTemplateExpr {
        &self.if_expr
    }

    pub fn else_expr(&self) -> Option<&ElseTemplateExpr> {
        self.else_expr.as_ref()
    }

    pub fn endif_expr(&self) -> &EndifTemplateExpr {
        &self.endif_expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.if_expr.despan(input);

        if let Some(else_expr) = &mut self.else_expr {
            else_expr.despan(input);
        }

        self.endif_expr.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfTemplateExpr {
    preamble: RawString,
    cond_expr: Expression,
    template: Template,
    strip: Strip,
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

    pub fn cond_expr(&self) -> &Expression {
        &self.cond_expr
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
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
    preamble: RawString,
    trailing: RawString,
    template: Template,
    strip: Strip,
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

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
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
    preamble: RawString,
    trailing: RawString,
    strip: Strip,
}

impl EndifTemplateExpr {
    pub fn new() -> EndifTemplateExpr {
        EndifTemplateExpr::default()
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForDirective {
    for_expr: ForTemplateExpr,
    endfor_expr: EndforTemplateExpr,
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

    pub fn for_expr(&self) -> &ForTemplateExpr {
        &self.for_expr
    }

    pub fn endfor_expr(&self) -> &EndforTemplateExpr {
        &self.endfor_expr
    }

    pub(crate) fn despan(&mut self, input: &str) {
        self.for_expr.despan(input);
        self.endfor_expr.despan(input);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForTemplateExpr {
    preamble: RawString,
    key_var: Option<Decorated<Ident>>,
    value_var: Decorated<Ident>,
    collection_expr: Expression,
    template: Template,
    strip: Strip,
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

    pub fn key_var(&self) -> Option<&Decorated<Ident>> {
        self.key_var.as_ref()
    }

    pub fn value_var(&self) -> &Decorated<Ident> {
        &self.value_var
    }

    pub fn collection_expr(&self) -> &Expression {
        &self.collection_expr
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
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
    preamble: RawString,
    trailing: RawString,
    strip: Strip,
}

impl EndforTemplateExpr {
    pub fn new() -> EndforTemplateExpr {
        EndforTemplateExpr::default()
    }

    pub fn strip(&self) -> Strip {
        self.strip
    }

    pub fn set_strip(&mut self, strip: Strip) {
        self.strip = strip;
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
