//! Format HCL language items.

mod fragments;

use self::fragments::{DecorFormatter, ModifyDecor, Padding, Trim};
use crate::expr::{
    Array, Expression, FuncArgs, Object, ObjectKeyMut, ObjectValue, ObjectValueAssignment,
    ObjectValueTerminator,
};
use crate::repr::Decorate;
use crate::structure::{Attribute, Block, BlockBody, BlockLabel, Body, Structure};
use crate::visit_mut::{
    visit_body_mut, visit_expr_mut, visit_object_mut, visit_structure_mut, VisitMut,
};
use crate::RawString;
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
    skip_first_line: bool,
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
            skip_first_line: false,
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
        self.skip_first_line = false;
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
    fn indent_next_line(&mut self, yes: bool) {
        self.indent.skip_first_line = !yes;
    }

    fn indent(&mut self) -> IndentGuard<'_> {
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
        P: FnOnce(DecorFormatter<Option<RawString>>) -> DecorFormatter<Option<RawString>>,
        S: FnOnce(DecorFormatter<Option<RawString>>) -> DecorFormatter<Option<RawString>>,
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
        P: FnOnce(DecorFormatter<Option<RawString>>) -> DecorFormatter<Option<RawString>>,
        F: FnOnce(&mut Formatter, &mut V),
        S: FnOnce(DecorFormatter<Option<RawString>>) -> DecorFormatter<Option<RawString>>,
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
        self.indent_next_line(true);
        self.visit_decorated(
            node,
            |prefix| prefix,
            |fmt, node| visit_structure_mut(fmt, node),
            |suffix| suffix.trim(Trim::Start).padding(Padding::Start),
        );
    }

    fn visit_attr_mut(&mut self, node: &'ast mut Attribute) {
        self.visit_decor(
            &mut node.key,
            |prefix| prefix.inline().trim(Trim::Both).padding(Padding::End),
            |suffix| suffix.inline().trim(Trim::Both).padding(Padding::Both),
        );
        self.indent_next_line(false);
        self.visit_decorated(
            &mut node.value,
            |prefix| prefix.inline().trim(Trim::Both).padding(Padding::Both),
            |fmt, node| visit_expr_mut(fmt, node),
            |suffix| suffix.inline().trim(Trim::Both).padding(Padding::Start),
        );
    }

    fn visit_block_mut(&mut self, node: &'ast mut Block) {
        self.visit_decor(
            &mut node.ident,
            |prefix| prefix.inline().trim(Trim::Both).padding(Padding::End),
            |suffix| suffix.inline().trim(Trim::Both).padding(Padding::Both),
        );
        for label in &mut node.labels {
            self.visit_block_label_mut(label);
        }
        self.visit_block_body_mut(&mut node.body);
    }

    fn visit_block_label_mut(&mut self, node: &'ast mut BlockLabel) {
        self.visit_decor(
            node,
            |prefix| prefix.inline().trim(Trim::Both).padding(Padding::End),
            |suffix| suffix.inline().trim(Trim::Both).padding(Padding::Both),
        )
    }

    fn visit_expr_mut(&mut self, node: &'ast mut Expression) {
        self.visit(node, |fmt, node| visit_expr_mut(fmt, node));
    }

    fn visit_array_mut(&mut self, node: &'ast mut Array) {
        if is_multiline_array(node) {
            self.visit(node, |fmt, node| {
                let mut guard = fmt.indent();
                make_multiline_exprs(&mut guard, node.iter_mut());
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| {
                        prefix
                            .padding(if i == 0 { Padding::End } else { Padding::Both })
                            .trim(Trim::Both)
                    },
                    |fmt, value| visit_expr_mut(fmt, value),
                    |suffix| suffix.trim(Trim::Both).padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing
                .modify()
                .trim(Trim::Both)
                .padding(padding)
                .format(self);
        }
    }

    fn visit_object_mut(&mut self, node: &'ast mut Object) {
        if is_multiline_object(node) {
            self.visit(node, |fmt, node| {
                let mut guard = fmt.indent();
                make_multiline_items(&mut guard, node.iter_mut());
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            visit_object_mut(self, node);

            node.trailing
                .modify()
                .trim(Trim::Both)
                .padding(Padding::Both)
                .format(self);
        }
    }

    fn visit_object_key_mut(&mut self, mut node: ObjectKeyMut<'ast>) {
        self.visit_decor(
            &mut node,
            |prefix| prefix.padding(Padding::End),
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
        if is_multiline_func_args(node) {
            self.visit(node, |fmt, node| {
                let mut guard = fmt.indent();
                make_multiline_exprs(&mut guard, node.iter_mut());
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                self.visit_decorated(
                    expr,
                    |prefix| {
                        prefix
                            .padding(if i == 0 { Padding::End } else { Padding::Both })
                            .trim(Trim::Both)
                    },
                    |fmt, value| visit_expr_mut(fmt, value),
                    |suffix| suffix.trim(Trim::Both).padding(Padding::Start),
                );
            }

            let padding = if node.trailing_comma() {
                Padding::Both
            } else {
                Padding::Start
            };

            node.trailing
                .modify()
                .trim(Trim::Both)
                .padding(padding)
                .format(self);
        }
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.indent_next_line(false);
                self.visit_decorated(
                    body,
                    |prefix| prefix.trim(Trim::Both).padding(Padding::Start),
                    |fmt, node| {
                        let mut guard = fmt.indent();
                        guard.visit_body_mut(node);
                        guard.indent_next_line(true);
                    },
                    |suffix| suffix.trim(Trim::Both).padding(Padding::Both),
                );
            }
            BlockBody::Oneline(body) => self.visit_oneline_body_mut(body),
        }
    }
}

fn make_multiline_exprs<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = &'a mut Expression>,
) {
    for expr in iter {
        fmt.visit_decorated(
            expr,
            |prefix| prefix.leading_newline().indent_empty_trailing_line(),
            |fmt, value| visit_expr_mut(fmt, value),
            |suffix| suffix.trim(Trim::End),
        );
    }
}

fn make_multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
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
    /* comment */,
    2,
    3 /* comment */,
    /* comment*/
    
    4,
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
