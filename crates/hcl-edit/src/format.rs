//! Format HCL language items.

use crate::expr::{Array, Expression, FuncArgs};
use crate::repr::{Decor, Decorate, Decorated};
use crate::structure::{Attribute, BlockBody, Body, Structure};
use crate::util::{dedent, indent_with};
use crate::visit_mut::{
    visit_array_mut, visit_body_mut, visit_expr_mut, visit_func_args_mut, visit_structure_mut,
    VisitMut,
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
    fn increase_indent(&mut self) -> &mut Self {
        self.indenter.increase();
        self
    }

    fn decrease_indent(&mut self) -> &mut Self {
        self.indenter.decrease();
        self
    }

    fn indent_next_line(&mut self, yes: bool) -> &mut Self {
        self.indenter.skip_first_line = !yes;
        self
    }

    fn indent(&self) -> String {
        self.indenter.prefix.repeat(self.indenter.level)
    }

    fn newline_indented(&self, s: &str) -> String {
        format!("\n{}{}", self.indent(), s)
    }

    fn indented<T, F>(&mut self, value: &mut T, f: F) -> &mut Self
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
        if !is_multiline_array(node) {
            return visit_array_mut(self, node);
        }

        self.increase_indent()
            .indented(node, |fmt, node| {
                for expr in node.iter_mut() {
                    visit_expr_mut(fmt, expr);
                    make_multiline_expr(fmt, expr);
                }
            })
            .decrease_indent();

        node.set_trailing(self.newline_indented(node.trailing().trim()));
    }

    fn visit_func_args_mut(&mut self, node: &'ast mut FuncArgs) {
        if !is_multiline_func_args(node) {
            return visit_func_args_mut(self, node);
        }

        self.increase_indent()
            .indented(node, |fmt, node| {
                for expr in node.iter_mut() {
                    visit_expr_mut(fmt, expr);
                    make_multiline_expr(fmt, expr);
                }
            })
            .decrease_indent();

        node.set_trailing(self.newline_indented(node.trailing().trim()));
    }

    fn visit_block_body_mut(&mut self, node: &'ast mut BlockBody) {
        match node {
            BlockBody::Multiline(body) => {
                self.indent_next_line(false).indented(body, |fmt, node| {
                    fmt.increase_indent().visit_body_mut(node);
                    fmt.decrease_indent().indent_next_line(true);
                });
            }
            BlockBody::Oneline(body) => self.visit_oneline_body_mut(body),
        }
    }
}

fn make_multiline_expr(fmt: &mut Formatter, expr: &mut Expression) {
    let decor = expr.decor_mut();
    let prefix = decor.take_prefix().unwrap_or_default();
    let suffix = decor.take_suffix().unwrap_or_default();
    *decor = Decor::new(fmt.newline_indented(prefix.trim()), suffix.trim());
}

fn is_multiline_expr(expr: &Expression) -> bool {
    let decor = expr.decor();

    decor.prefix().map_or(false, |p| p.contains('\n'))
        || decor.suffix().map_or(false, |p| p.contains('\n'))
}

fn is_multiline_array(array: &Array) -> bool {
    array.iter().any(is_multiline_expr) || array.trailing().contains('\n')
}

fn is_multiline_func_args(args: &FuncArgs) -> bool {
    args.iter().any(is_multiline_expr) || args.trailing().contains('\n')
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
  }

  array = [1, /* two */ 2, 3 , ]

  multiline_array = [
    1/* comment */,
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
      /* three */3
    ]
  )
}

/* some trailing comment */"#;

        let mut body = input.parse::<Body>().unwrap();
        body.default_format();

        assert_eq!(body.to_string(), expected);
    }
}
