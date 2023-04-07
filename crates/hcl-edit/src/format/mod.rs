//! Format HCL language items.

mod fragments;

use self::fragments::{ModifyDecor, Padding, Trim};
use crate::expr::{
    Array, Expression, FuncArgs, Object, ObjectKeyMut, ObjectValue, ObjectValueAssignment,
    ObjectValueTerminator,
};
use crate::repr::{Decorate, Decorated};
use crate::structure::{Attribute, BlockBody, Body, Structure};
use crate::visit_mut::{
    visit_body_mut, visit_expr_mut, visit_object_mut, visit_structure_mut, VisitMut,
};
use crate::Ident;
use hcl_primitives::InternalString;

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

impl Formatter {
    fn indent_next_line(&mut self, yes: bool) -> &mut Self {
        self.indent.skip_first_line = !yes;
        self
    }

    fn indented<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Formatter),
    {
        self.indent.increase();
        f(self);
        self.indent.decrease();
        self
    }

    fn indented_format_decor<F, T>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        self.indented(|fmt| {
            fmt.format_decor(value, f);
        })
    }

    fn format_decor<T, F>(&mut self, value: &mut T, f: F) -> &mut Self
    where
        T: Decorate + ?Sized,
        F: FnOnce(&mut Formatter, &mut T),
    {
        value.decor_mut().prefix.modify().format(self);
        f(self, value);
        value.decor_mut().suffix.modify().format(self);
        self
    }
}

impl<'ast> VisitMut<'ast> for Formatter {
    fn visit_body_mut(&mut self, node: &'ast mut Body) {
        self.format_decor(node, |fmt, node| visit_body_mut(fmt, node));
    }

    fn visit_structure_mut(&mut self, node: &'ast mut Structure) {
        self.indent_next_line(true)
            .format_decor(node, |fmt, node| visit_structure_mut(fmt, node));
    }

    fn visit_attr_mut(&mut self, node: &'ast mut Attribute) {
        self.visit_ident_mut(&mut node.key);
        self.indent_next_line(false);
        self.visit_expr_mut(&mut node.value);
    }

    fn visit_ident_mut(&mut self, node: &'ast mut Decorated<Ident>) {
        self.format_decor(node, |_, _| ());
    }

    fn visit_expr_mut(&mut self, node: &'ast mut Expression) {
        self.format_decor(node, |fmt, node| visit_expr_mut(fmt, node));
    }

    fn visit_array_mut(&mut self, node: &'ast mut Array) {
        if is_multiline_array(node) {
            self.indented_format_decor(node, |fmt, node| {
                make_multiline_exprs(fmt, node.iter_mut())
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                visit_expr_mut(self, expr);

                let decor = expr.decor_mut();
                let mut prefix = decor.prefix.modify();

                if i == 0 {
                    prefix.padding(Padding::End);
                } else {
                    prefix.padding(Padding::Both);
                }

                prefix.trim(Trim::TrailingWhitespace).format(self);

                decor
                    .suffix
                    .modify()
                    .trim(Trim::TrailingWhitespace)
                    .padding(Padding::Start)
                    .format(self);
            }

            node.trailing
                .modify()
                .trim(Trim::TrailingWhitespace)
                .padding(Padding::Both)
                .format(self);
        }
    }

    fn visit_object_mut(&mut self, node: &'ast mut Object) {
        if is_multiline_object(node) {
            self.indented_format_decor(node, |fmt, node| {
                make_multiline_items(fmt, node.iter_mut())
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            visit_object_mut(self, node);
            node.trailing
                .modify()
                .trim(Trim::TrailingWhitespace)
                .padding(Padding::Both)
                .format(self);
        }
    }

    fn visit_object_key_mut(&mut self, mut node: ObjectKeyMut<'ast>) {
        let decor = node.decor_mut();
        decor.prefix.modify().padding(Padding::End).format(self);
        decor
            .suffix
            .modify()
            .inline()
            .padding(Padding::Both)
            .format(self);
    }

    fn visit_object_value_mut(&mut self, node: &'ast mut ObjectValue) {
        node.set_assignment(ObjectValueAssignment::Equals);

        let decor = node.expr_mut().decor_mut();
        decor
            .prefix
            .modify()
            .inline()
            .padding(Padding::Both)
            .format(self);
        decor
            .suffix
            .modify()
            .inline()
            .padding(Padding::Start)
            .format(self);
    }

    fn visit_func_args_mut(&mut self, node: &'ast mut FuncArgs) {
        if is_multiline_func_args(node) {
            self.indented_format_decor(node, |fmt, node| {
                make_multiline_exprs(fmt, node.iter_mut())
            });
            node.trailing.modify().leading_newline().format(self);
        } else {
            for (i, expr) in node.iter_mut().enumerate() {
                visit_expr_mut(self, expr);

                let decor = expr.decor_mut();
                let mut prefix = decor.prefix.modify();

                if i == 0 {
                    prefix.padding(Padding::End);
                } else {
                    prefix.padding(Padding::Both);
                }

                prefix.trim(Trim::TrailingWhitespace).format(self);

                decor
                    .suffix
                    .modify()
                    .trim(Trim::TrailingWhitespace)
                    .padding(Padding::Start)
                    .format(self);
            }

            let trailing_comma = node.trailing_comma();
            let mut trailing = node.trailing.modify();

            if trailing_comma {
                trailing.padding(Padding::Both);
            } else {
                trailing.padding(Padding::Start);
            }

            trailing.trim(Trim::TrailingWhitespace).format(self);
        }
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.indent_next_line(false)
                    .format_decor(body, |fmt, node| {
                        fmt.indented(|fmt| fmt.visit_body_mut(node))
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
        decor
            .prefix
            .modify()
            .leading_newline()
            .indent_empty_trailing_line()
            .format(fmt);
        decor
            .suffix
            .modify()
            .trim(Trim::TrailingWhitespace)
            .format(fmt);
    }
}

fn make_multiline_items<'a>(
    fmt: &'a mut Formatter,
    iter: impl Iterator<Item = (ObjectKeyMut<'a>, &'a mut ObjectValue)>,
) {
    for (mut key, value) in iter {
        let key_decor = key.decor_mut();
        key_decor
            .prefix
            .modify()
            .leading_newline()
            .padding(Padding::End)
            .format(fmt);
        key_decor
            .suffix
            .modify()
            .inline()
            .padding(Padding::Both)
            .format(fmt);

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
