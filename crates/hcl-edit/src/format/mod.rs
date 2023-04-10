//! Format HCL language items.

mod fragments;

use self::fragments::{DecorFormatter, ModifyDecor, Padding};
use crate::expr::{
    Array, Expression, FuncArgs, Object, ObjectKey, ObjectKeyMut, ObjectValue,
    ObjectValueAssignment, ObjectValueTerminator,
};
use crate::repr::Decorate;
use crate::structure::{Attribute, Block, BlockBody, BlockLabel, Body, Structure};
use crate::visit_mut::{
    visit_body_mut, visit_expr_mut, visit_object_mut, visit_structure_mut, VisitMut,
};
use hcl_primitives::InternalString;
use std::ops;

/// Builds a [`Formatter`].
#[derive(Default)]
pub struct FormatterBuilder {
    indent: Indent,
}

impl FormatterBuilder {
    /// Sets the indent.
    pub fn indent(&mut self, indent: Indent) -> &mut Self {
        self.indent = indent;
        self
    }

    /// Builds a [`Formatter`] from the builder's configuration.
    pub fn build(&self) -> Formatter {
        Formatter {
            indent: self.indent.clone(),
        }
    }
}

/// A configurable formatter for HCL language items.
#[derive(Debug, Clone, Default)]
pub struct Formatter {
    indent: Indent,
}

// Public API.
impl Formatter {
    /// Resets the formatter state.
    pub fn reset(&mut self) {
        self.indent.reset();
    }

    /// Creates a builder for configuring a [`Formatter`].
    pub fn builder() -> FormatterBuilder {
        FormatterBuilder::default()
    }
}

/// Applies indentation.
#[derive(Debug, Clone)]
pub struct Indent {
    level: usize,
    prefix: InternalString,
    indent_first_line: bool,
}

impl Default for Indent {
    fn default() -> Self {
        Indent::new("  ")
    }
}

impl Indent {
    /// Creates a new `Indent` from a prefix.
    pub fn new(prefix: impl Into<InternalString>) -> Indent {
        Indent {
            level: 0,
            prefix: prefix.into(),
            indent_first_line: true,
        }
    }

    /// Creates a new `Indent` from a number of spaces.
    pub fn spaces(n: usize) -> Indent {
        Indent::new(" ".repeat(n))
    }

    fn increase(&mut self) {
        self.level += 1;
    }

    fn decrease(&mut self) {
        self.level -= 1;
    }

    fn reset(&mut self) {
        self.level = 0;
        self.indent_first_line = true;
    }

    fn prefix(&self) -> String {
        self.prefix.repeat(self.level)
    }
}

struct IndentGuard<'a> {
    formatter: &'a mut Formatter,
}

impl ops::Deref for IndentGuard<'_> {
    type Target = Formatter;

    fn deref(&self) -> &Self::Target {
        self.formatter
    }
}

impl ops::DerefMut for IndentGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.formatter
    }
}

impl Drop for IndentGuard<'_> {
    fn drop(&mut self) {
        self.formatter.indent.decrease();
    }
}

impl Formatter {
    fn indented(&mut self) -> IndentGuard<'_> {
        self.indent.increase();
        IndentGuard { formatter: self }
    }

    fn visit<V, F>(&mut self, value: &mut V, f: F)
    where
        V: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut V),
    {
        self.visit_decorated(value, |prefix| prefix, f, |suffix| suffix);
    }

    fn visit_decor<V, P, S>(&mut self, value: &mut V, modify_prefix: P, modify_suffix: S)
    where
        V: Decorate + ?Sized,
        P: FnOnce(DecorFormatter) -> DecorFormatter,
        S: FnOnce(DecorFormatter) -> DecorFormatter,
    {
        self.visit_decorated(value, modify_prefix, |_, _| (), modify_suffix)
    }

    fn visit_decorated<V, P, F, S>(
        &mut self,
        value: &mut V,
        modify_prefix: P,
        f: F,
        modify_suffix: S,
    ) where
        V: Decorate + ?Sized,
        P: FnOnce(DecorFormatter) -> DecorFormatter,
        F: FnOnce(&mut Formatter, &mut V),
        S: FnOnce(DecorFormatter) -> DecorFormatter,
    {
        modify_prefix(value.decor_mut().prefix.modify()).format(self);
        f(self, value);
        modify_suffix(value.decor_mut().suffix.modify()).format(self);
    }
}

impl<'ast> VisitMut<'ast> for Formatter {
    fn visit_body_mut(&mut self, node: &'ast mut Body) {
        self.visit(node, |fmt, node| visit_body_mut(fmt, node));
    }

    fn visit_structure_mut(&mut self, node: &'ast mut Structure) {
        self.visit_decorated(
            node,
            |prefix| prefix.indent_first_line(true).padding(Padding::End),
            |fmt, node| visit_structure_mut(fmt, node),
            |suffix| suffix.padding(Padding::Start),
        );
    }

    fn visit_attr_mut(&mut self, node: &'ast mut Attribute) {
        self.visit_decor(
            &mut node.key,
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );
        self.visit_decorated(
            &mut node.value,
            |prefix| {
                prefix
                    .inline()
                    .indent_first_line(false)
                    .padding(Padding::Both)
            },
            |fmt, node| visit_expr_mut(fmt, node),
            |suffix| suffix.inline().padding(Padding::Start),
        );
    }

    fn visit_block_mut(&mut self, node: &'ast mut Block) {
        self.visit_decor(
            &mut node.ident,
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );
        for label in &mut node.labels {
            self.visit_block_label_mut(label);
        }
        self.visit_block_body_mut(&mut node.body);
    }

    fn visit_block_label_mut(&mut self, node: &'ast mut BlockLabel) {
        self.visit_decor(
            node,
            |prefix| prefix.inline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        )
    }

    fn visit_expr_mut(&mut self, node: &'ast mut Expression) {
        self.visit(node, |fmt, node| visit_expr_mut(fmt, node));
    }

    fn visit_array_mut(&mut self, node: &'ast mut Array) {
        if has_multiline_elements(node.iter()) || node.trailing.is_multiline() {
            multiline_exprs(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| prefix.padding(if i == 0 { Padding::End } else { Padding::Both }),
                    |fmt, value| visit_expr_mut(fmt, value),
                    |suffix| suffix.padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing.modify().padding(padding).format(self);
        }
    }

    fn visit_object_mut(&mut self, node: &'ast mut Object) {
        if has_multiline_items(node.iter()) || node.trailing.is_multiline() {
            multiline_items(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            visit_object_mut(self, node);

            node.trailing.modify().padding(Padding::Both).format(self);
        }
    }

    fn visit_object_key_mut(&mut self, mut node: ObjectKeyMut<'ast>) {
        self.visit_decor(
            &mut node,
            |prefix| prefix.padding(Padding::Both),
            |suffix| suffix.inline().padding(Padding::Both),
        );
    }

    fn visit_object_value_mut(&mut self, node: &'ast mut ObjectValue) {
        node.set_assignment(ObjectValueAssignment::Equals);

        self.visit_decorated(
            node.expr_mut(),
            |prefix| prefix.inline().padding(Padding::Both),
            |fmt, node| visit_expr_mut(fmt, node),
            |suffix| suffix.inline().padding(Padding::Start),
        );
    }

    fn visit_func_args_mut(&mut self, node: &'ast mut FuncArgs) {
        if has_multiline_elements(node.iter()) || node.trailing.is_multiline() {
            multiline_exprs(self, node.iter_mut());
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| prefix.padding(if i == 0 { Padding::End } else { Padding::Both }),
                    |fmt, value| visit_expr_mut(fmt, value),
                    |suffix| suffix.padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing.modify().padding(padding).format(self);
        }
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.visit_decorated(
                    body,
                    |prefix| prefix.indent_first_line(false).padding(Padding::Start),
                    |fmt, node| {
                        let mut fmt = fmt.indented();
                        visit_body_mut(&mut *fmt, node)
                    },
                    |suffix| suffix.indent_first_line(true).padding(Padding::Both),
                );
            }
            BlockBody::Oneline(body) => self.visit_oneline_body_mut(body),
        }
    }
}

fn multiline_exprs<'a>(fmt: &'a mut Formatter, iter: impl Iterator<Item = &'a mut Expression>) {
    let mut fmt = fmt.indented();

    for expr in iter {
        fmt.visit_decorated(
            expr,
            |prefix| prefix.leading_newline().padding(Padding::End),
            |fmt, value| visit_expr_mut(fmt, value),
            |suffix| suffix.padding(Padding::Start),
        );
    }
}

fn multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
    let mut fmt = fmt.indented();

    for (mut key, value) in iter {
        fmt.visit_decor(
            &mut key,
            |prefix| prefix.leading_newline().padding(Padding::End),
            |suffix| suffix.inline().padding(Padding::Both),
        );

        value.set_terminator(ObjectValueTerminator::None);

        fmt.visit_object_value_mut(value);
    }
}

fn has_multiline_items<'a>(
    mut iter: impl Iterator<Item = (&'a ObjectKey, &'a ObjectValue)>,
) -> bool {
    iter.any(|(k, v)| k.decor().is_multiline() || v.expr().decor().is_multiline())
}

fn has_multiline_elements<'a, T>(mut iter: impl Iterator<Item = &'a T>) -> bool
where
    T: Decorate + 'a,
{
    iter.any(|v| v.decor().is_multiline())
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
block  "label"  {  # comment
    // comment
attr1 = "value"
    attr2 = 42

// another comment
nested_block {
foo = 1  # foo comment

    object = { foo :bar, baz= qux,  }

    multiline_object = { foo = bar/*comment */,
     /* comment */baz = qux, one =/*comment*/1, multi = 42 /*
  multiline comment */
    // another
      # and another
two:2 }
}

    array = [1,     /* two */ 2, 3 ,      ]

      multiline_array    =    [

      1
      /* comment */
    ,
    2,
        3 /* comment */,
  /* comment*/

  4

  ,
        ]

    bar =   func(1, [
        2, 3])

    baz  = func(
     1, [
        2, /* three */ 3])

qux = func( 1  , /*two*/3  ...  )
  }

  /* some trailing comment */"#;

        let expected = r#"
// comment
block "label" { # comment
  // comment
  attr1 = "value"
  attr2 = 42

  // another comment
  nested_block {
    foo = 1 # foo comment

    object = { foo = bar, baz = qux, }

    multiline_object = {
      foo = bar /*comment */
      /* comment */ baz = qux
      one = /*comment*/ 1
      multi = 42 /*
  multiline comment */
      // another
      # and another
      two = 2
    }
  }

  array = [1, /* two */ 2, 3, ]

  multiline_array = [

    1
    /* comment */
    ,
    2,
    3 /* comment */,
    /* comment*/

    4

    ,
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

  qux = func(1, /*two*/ 3...)
}

/* some trailing comment */"#;

        let mut body = input.parse::<Body>().unwrap();
        body.default_format();

        assert_eq!(body.to_string(), expected);
    }
}
