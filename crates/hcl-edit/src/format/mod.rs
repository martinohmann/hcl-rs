//! Format HCL language items.

mod fragments;

use self::fragments::{DecorKind, ParseDecor};
use crate::expr::{
    Array, Expression, FuncArgs, Object, ObjectKeyMut, ObjectValue, ObjectValueAssignment,
    ObjectValueTerminator,
};
use crate::repr::{Decor, Decorate, Decorated};
use crate::structure::{Attribute, BlockBody, Body, Structure};
use crate::visit_mut::{
    visit_body_mut, visit_expr_mut, visit_object_mut, visit_structure_mut, VisitMut,
};
use crate::Ident;
use hcl_primitives::InternalString;

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
pub(crate) struct Indenter {
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

    fn prefix(&self) -> String {
        self.prefix.repeat(self.level)
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

    fn descend_indented<F, T>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        self.indent().indented(value, f).dedent()
    }

    fn indent_next_line(&mut self, yes: bool) -> &mut Self {
        self.indenter.skip_first_line = !yes;
        self
    }

    fn indented<T, F>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        self.indent_prefix(value.decor_mut(), DecorKind::Multiline);
        f(self, value);
        self.indent_suffix(value.decor_mut(), DecorKind::Multiline);
        self
    }

    fn indent_prefix(&mut self, decor: &mut Decor, kind: DecorKind) {
        decor.set_prefix(decor.prefix().parse_as(kind).format(self));
    }

    fn indent_suffix(&mut self, decor: &mut Decor, kind: DecorKind) {
        decor.set_suffix(decor.suffix().parse_as(kind).format(self));
    }
}

impl<'ast> VisitMut<'ast> for Formatter {
    fn visit_body_mut(&mut self, node: &'ast mut Body) {
        self.indented(node, |fmt, node| visit_body_mut(fmt, node));
    }

    fn visit_structure_mut(&mut self, node: &'ast mut Structure) {
        self.indent_next_line(true)
            .indented(node, |fmt, node| visit_structure_mut(fmt, node));
    }

    fn visit_attr_mut(&mut self, node: &'ast mut Attribute) {
        self.visit_ident_mut(&mut node.key);
        self.indent_next_line(false);
        self.visit_expr_mut(&mut node.value);
    }

    fn visit_ident_mut(&mut self, node: &'ast mut Decorated<Ident>) {
        self.indented(node, |_, _| ());
    }

    fn visit_expr_mut(&mut self, node: &'ast mut Expression) {
        self.indented(node, |fmt, node| visit_expr_mut(fmt, node));
    }

    fn visit_array_mut(&mut self, node: &'ast mut Array) {
        if is_multiline_array(node) {
            self.descend_indented(node, |fmt, node| make_multiline_exprs(fmt, node.iter_mut()));
            let trailing = node
                .trailing()
                .parse_multiline()
                .leading_newline()
                .format(self);
            node.set_trailing(trailing);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                visit_expr_mut(self, expr);

                let decor = expr.decor_mut();
                let prefix = decor.prefix();
                let mut parsed_prefix = prefix.parse_multiline();
                parsed_prefix.trim_trailing_whitespace();

                if i == 0 {
                    parsed_prefix.space_padded_end();
                } else {
                    parsed_prefix.space_padded();
                }

                decor.set_prefix(parsed_prefix.format(self));

                let suffix = decor
                    .suffix()
                    .parse_multiline()
                    .trim_trailing_whitespace()
                    .space_padded_start()
                    .format(self);
                decor.set_suffix(suffix);
            }

            let trailing = node
                .trailing()
                .parse_multiline()
                .trim_trailing_whitespace()
                .space_padded()
                .format(self);
            node.set_trailing(trailing);
        }
    }

    fn visit_object_mut(&mut self, node: &'ast mut Object) {
        if is_multiline_object(node) {
            self.descend_indented(node, |fmt, node| make_multiline_items(fmt, node.iter_mut()));
            let trailing = node
                .trailing()
                .parse_multiline()
                .leading_newline()
                .format(self);
            node.set_trailing(trailing);
        } else {
            visit_object_mut(self, node);
            let trailing = node
                .trailing()
                .parse_multiline()
                .trim_trailing_whitespace()
                .space_padded()
                .format(self);
            node.set_trailing(trailing);
        }
    }

    fn visit_object_key_mut(&mut self, mut node: ObjectKeyMut<'ast>) {
        let decor = node.decor_mut();
        let prefix = decor
            .prefix()
            .parse_multiline()
            .space_padded_end()
            .format(self);
        decor.set_prefix(prefix);
        let suffix = decor.suffix().parse_inline().space_padded().format(self);
        decor.set_suffix(suffix);
    }

    fn visit_object_value_mut(&mut self, node: &'ast mut ObjectValue) {
        node.set_assignment(ObjectValueAssignment::Equals);

        let decor = node.expr_mut().decor_mut();
        let prefix = decor.prefix().parse_inline().space_padded().format(self);
        decor.set_prefix(prefix);

        let suffix = decor
            .suffix()
            .parse_inline()
            .space_padded_start()
            .format(self);
        decor.set_suffix(suffix);
    }

    fn visit_func_args_mut(&mut self, node: &'ast mut FuncArgs) {
        if is_multiline_func_args(node) {
            self.descend_indented(node, |fmt, node| make_multiline_exprs(fmt, node.iter_mut()));
            let trailing = node
                .trailing()
                .parse_multiline()
                .leading_newline()
                .format(self);
            node.set_trailing(trailing);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                visit_expr_mut(self, expr);

                let decor = expr.decor_mut();
                let prefix = decor.prefix();
                let mut parsed_prefix = prefix.parse_multiline();
                parsed_prefix.trim_trailing_whitespace();

                if i == 0 {
                    parsed_prefix.space_padded_end();
                } else {
                    parsed_prefix.space_padded();
                }

                decor.set_prefix(parsed_prefix.format(self));

                let suffix = decor
                    .suffix()
                    .parse_multiline()
                    .trim_trailing_whitespace()
                    .space_padded_start()
                    .format(self);
                decor.set_suffix(suffix);
            }

            let mut parsed_trailing = node.trailing().parse_multiline();
            parsed_trailing.trim_trailing_whitespace();

            if node.trailing_comma() {
                parsed_trailing.space_padded();
            } else {
                parsed_trailing.space_padded_start();
            }

            node.set_trailing(parsed_trailing.format(self));
        }
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.indent_next_line(false).indented(body, |fmt, node| {
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
    for expr in iter {
        visit_expr_mut(fmt, expr);

        let decor = expr.decor_mut();
        let prefix = decor
            .prefix()
            .parse_multiline()
            .leading_newline()
            .indent_empty_trailing_line()
            .format(fmt);
        decor.set_prefix(prefix);
        let suffix = decor
            .suffix()
            .parse_multiline()
            .trim_trailing_whitespace()
            .format(fmt);
        decor.set_suffix(suffix);
    }
}

fn make_multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
    for (mut key, value) in iter {
        let key_decor = key.decor_mut();
        let prefix = key_decor
            .prefix()
            .parse_multiline()
            .leading_newline()
            .space_padded_end()
            .format(fmt);
        key_decor.set_prefix(prefix);
        let suffix = key_decor.suffix().parse_inline().space_padded().format(fmt);
        key_decor.set_suffix(suffix);

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
block {  # comment
    // comment
attr1 = "value"
    attr2 = 42

// another comment
nested_block {
foo = 1 # foo comment

    object = { foo :bar, baz= qux,  }

    multiline_object = { foo = bar/*comment */,
     /* comment */baz = qux, one =/*comment*/1, multi = 42 /*
  multiline comment */
    // another
      # and another
two:2 }
}

    array = [1, /* two */ 2, 3 ,      ]

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
        2, 3])

    baz = func(
     1, [
        2, /* three */ 3])

qux = func( 1  , /*two*/3  ...  )
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
