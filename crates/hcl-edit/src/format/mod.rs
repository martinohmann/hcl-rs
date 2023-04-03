//! Format HCL language items.

mod normalize;

use self::normalize::Normalizer;
use crate::expr::{
    Array, Expression, FuncArgs, Object, ObjectKeyMut, ObjectValue, ObjectValueAssignment,
    ObjectValueTerminator,
};
use crate::repr::{Decorate, Decorated};
use crate::structure::{Attribute, BlockBody, Body, Structure};
use crate::util::{dedent, indent_with};
use crate::visit_mut::{
    visit_array_mut, visit_body_mut, visit_expr_mut, visit_func_args_mut, visit_object_mut,
    visit_object_value_mut, visit_structure_mut, VisitMut,
};
use crate::Ident;
use hcl_primitives::InternalString;
use std::borrow::Cow;

/// Builds a [`Formatter`].
pub struct FormatterBuilder {
    indent: usize,
}

impl Default for FormatterBuilder {
    fn default() -> Self {
        FormatterBuilder { indent: 2 }
    }
}

impl FormatterBuilder {
    /// Set the indent.
    pub fn indent(&mut self, indent: usize) -> &mut Self {
        self.indent = indent;
        self
    }

    /// Builds a [`Formatter`] from the builder's configuration.
    pub fn build(&self) -> Formatter {
        Formatter {
            indenter: Indenter::new("  ".repeat(self.indent)),
        }
    }
}

/// A configurable formatter for HCL language items.
#[derive(Debug, Clone, Default)]
pub struct Formatter {
    indenter: Indenter,
}

// Public API.
impl Formatter {
    /// Resets the formatter state.
    pub fn reset(&mut self) {}

    /// Creates a builder for configuring a [`Formatter`].
    pub fn builder() -> FormatterBuilder {
        FormatterBuilder::default()
    }
}

#[derive(Debug, Clone)]
struct Indenter {
    level: usize,
    prefix: InternalString,
    skip_first_line: bool,
}

impl Default for Indenter {
    fn default() -> Self {
        Indenter::new("  ")
    }
}

impl Indenter {
    fn new(prefix: impl Into<InternalString>) -> Indenter {
        Indenter {
            level: 0,
            prefix: prefix.into(),
            skip_first_line: false,
        }
    }

    fn increase(&mut self) {
        self.level += 1;
    }

    fn decrease(&mut self) {
        self.level -= 1;
    }

    fn reindent<'a>(&mut self, s: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
        let skip_first_line = dbg!(self.skip_first_line);
        let dedented = dbg!(dedent(s, skip_first_line));
        self.skip_first_line = !dedented.ends_with('\n');

        if self.level == 0 {
            dedented
        } else {
            let prefix = self.prefix.repeat(self.level);
            if dedented.is_empty() {
                if skip_first_line {
                    dedented
                } else {
                    Cow::Owned(prefix)
                }
            } else {
                indent_with(dedented, &prefix, skip_first_line)
            }
        }
    }

    fn reindent_prefix<T>(&mut self, value: &mut T)
    where
        T: Decorate + ?Sized,
    {
        let decor = value.decor_mut();

        if let Some(prefix) = decor.prefix() {
            decor.set_prefix(dbg!(self.reindent(prefix)));
        } else if !self.skip_first_line {
            if self.level > 0 {
                decor.set_prefix(self.prefix.repeat(self.level));
            }
            self.skip_first_line = true;
        }
    }

    fn reindent_suffix<T>(&mut self, value: &mut T)
    where
        T: Decorate + ?Sized,
    {
        let decor = value.decor_mut();

        if let Some(suffix) = decor.suffix() {
            decor.set_suffix(self.reindent(suffix));
        }
    }
}

impl Formatter {
    fn indent(&mut self) -> &mut Self {
        self.indenter.increase();
        self
    }

    fn dedent(&mut self) -> &mut Self {
        self.indenter.decrease();
        self
    }

    fn descend<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Formatter),
    {
        self.indent();
        f(self);
        self.dedent();
        self
    }

    fn descend_indent_decor<F, T>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        self.indent().indent_decor(value, f).dedent()
    }

    fn indent_next_line(&mut self, yes: bool) -> &mut Self {
        self.indenter.skip_first_line = !yes;
        self
    }

    fn newline_indented<S: std::fmt::Display>(&self, s: S) -> String {
        let indent = self.indenter.prefix.repeat(self.indenter.level);
        format!("\n{}{}", indent, s)
    }

    fn newline_indent(&self) -> String {
        let indent = self.indenter.prefix.repeat(self.indenter.level);
        format!("\n{}", indent)
    }

    fn indent_decor<T, F>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        self.indenter.reindent_prefix(value);
        f(self, value);
        self.indenter.reindent_suffix(value);
        self
    }
}

impl<'ast> VisitMut<'ast> for Formatter {
    fn visit_body_mut(&mut self, node: &'ast mut Body) {
        self.indent_decor(node, |fmt, node| visit_body_mut(fmt, node));
    }

    fn visit_structure_mut(&mut self, node: &'ast mut Structure) {
        self.indent_next_line(true)
            .indent_decor(node, |fmt, node| visit_structure_mut(fmt, node));
    }

    fn visit_attr_mut(&mut self, node: &'ast mut Attribute) {
        self.visit_ident_mut(&mut node.key);
        self.indent_next_line(false);
        self.visit_expr_mut(&mut node.value);
    }

    fn visit_ident_mut(&mut self, node: &'ast mut Decorated<Ident>) {
        self.indent_decor(node, |_, _| ());
    }

    fn visit_expr_mut(&mut self, node: &'ast mut Expression) {
        self.indent_decor(node, |fmt, node| visit_expr_mut(fmt, node));
    }

    fn visit_array_mut(&mut self, node: &'ast mut Array) {
        if is_multiline_array(node) {
            self.descend_indent_decor(node, |fmt, node| make_multiline_exprs(fmt, node.iter_mut()));
            node.set_trailing(
                self.newline_indented(
                    Normalizer::new()
                        .multiline()
                        .normalize_raw(node.trailing())
                        .unwrap_or_default()
                        .as_str(),
                ),
            );
        } else {
            visit_array_mut(self, node);
        }
    }

    fn visit_object_mut(&mut self, node: &'ast mut Object) {
        if is_multiline_object(node) {
            self.descend_indent_decor(node, |fmt, node| make_multiline_items(fmt, node.iter_mut()));
            node.set_trailing(
                self.newline_indented(
                    Normalizer::new()
                        .multiline()
                        .normalize_raw(node.trailing())
                        .unwrap_or_default()
                        .as_str(),
                ),
            );
        } else {
            visit_object_mut(self, node);
        }
    }

    fn visit_func_args_mut(&mut self, node: &'ast mut FuncArgs) {
        if is_multiline_func_args(node) {
            self.descend_indent_decor(node, |fmt, node| make_multiline_exprs(fmt, node.iter_mut()));
            node.set_trailing(
                self.newline_indented(
                    Normalizer::new()
                        .multiline()
                        .normalize_raw(node.trailing())
                        .unwrap_or_default()
                        .as_str(),
                ),
            );
        } else {
            visit_func_args_mut(self, node);
        }
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.indent_next_line(false)
                    .indent_decor(body, |fmt, node| {
                        fmt.descend(|fmt| fmt.visit_body_mut(node))
                            .indent_next_line(true);
                    });
            }
            BlockBody::Oneline(body) => self.visit_oneline_body_mut(body),
        }
    }
}

fn make_multiline_exprs<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = &'a mut Expression>,
) {
    let prefix_normalizer = Normalizer::new().trailing_space().multiline();
    let suffix_normalizer = Normalizer::new().leading_space().multiline();

    for expr in iter {
        visit_expr_mut(fmt, expr);

        let decor = expr.decor_mut();

        let prefix = decor
            .take_prefix()
            .and_then(|prefix| prefix_normalizer.normalize_raw(&prefix))
            .map(|normalized| fmt.newline_indented(normalized.as_str()))
            .unwrap_or_else(|| fmt.newline_indent());
        decor.set_prefix(prefix);

        suffix_normalizer.normalize_decor_suffix(decor);
    }
}

fn make_multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
    let key_prefix_normalizer = Normalizer::new().multiline().trailing_space();
    let prefix_normalizer = Normalizer::new().trailing_space();
    let suffix_normalizer = Normalizer::new().leading_space();

    for (mut key, value) in iter {
        visit_object_value_mut(fmt, value);

        value.set_assignment(ObjectValueAssignment::Equals);
        value.set_terminator(ObjectValueTerminator::None);

        let key_decor = key.decor_mut();

        let key_prefix = key_decor
            .take_prefix()
            .and_then(|prefix| key_prefix_normalizer.normalize_raw(&prefix))
            .map(|normalized| fmt.newline_indented(normalized.as_str()))
            .unwrap_or_else(|| fmt.newline_indent());
        key_decor.set_prefix(key_prefix);

        suffix_normalizer.normalize_decor_suffix(key_decor);

        let value_decor = value.expr_mut().decor_mut();
        prefix_normalizer.normalize_decor_prefix(value_decor);
        suffix_normalizer.normalize_decor_suffix(value_decor);
    }
}

fn has_multiline_decor<T>(value: &T) -> bool
where
    T: Decorate + ?Sized,
{
    value.decor().is_multiline()
}

fn is_multiline_object(object: &Object) -> bool {
    object
        .iter()
        .any(|(k, v)| has_multiline_decor(k) || has_multiline_decor(v.expr()))
        || object.trailing().is_multiline()
}

fn is_multiline_array(array: &Array) -> bool {
    array.iter().any(has_multiline_decor) || array.trailing().is_multiline()
}

fn is_multiline_func_args(args: &FuncArgs) -> bool {
    args.iter().any(has_multiline_decor) || args.trailing().is_multiline()
}

#[cfg(test)]
mod tests {
    use crate::repr::Format;
    use crate::structure::Body;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_format_body() {
        let input = r#"
    // comment
block {  # comment
    // comment
attr1 = "value"
    attr2 = 42

// another comment
nested_block {
foo = 1 # comment

    object = { foo =bar, baz = qux,  }

    multiline_object = { foo = bar /*comment */,
     /* comment */ baz = qux, one =1, two:2 }
}

    array = [1, /* two */ 2, 3 , ]

      multiline_array = [
      1
      /* comment */
      ,
    2,
        3,
        ]

    bar = func(1, [
        2, 3])

    baz = func(
     1, [
        2, /* three */ 3])
  }

  /* some trailing comment */"#;

        let expected = r#"
// comment
block {  # comment
  // comment
  attr1 = "value"
  attr2 = 42

  // another comment
  nested_block {
    foo = 1 # comment

    object = { foo =bar, baz = qux,  }

    multiline_object = {
      foo = bar /*comment */
      /* comment */ baz = qux
      one = 1
      two = 2
    }
  }

  array = [1, /* two */ 2, 3 , ]

  multiline_array = [
    1 /* comment */,
    2,
    3,
  ]

  bar = func(1, [
    2,
    3
  ])

  baz = func(
    1,
    [
      2,
      /* three */ 3
    ]
  )
}

/* some trailing comment */"#;

        let mut body = input.parse::<Body>().unwrap();
        body.default_format();

        assert_eq!(body.to_string(), expected);
    }
}
