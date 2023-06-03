//! Format HCL language items.

mod decor;
#[cfg(test)]
mod tests;
mod visit;

use self::decor::{DecorFormatter, ModifyDecor};
use crate::repr::{Decor, Decorate};
use hcl_primitives::InternalString;
use std::ops;

/// A trait for objects which can be formatted.
pub trait Format {
    /// Formats an object.
    fn format_with(&mut self, formatter: Formatter);

    /// Applies the default format to an object.
    fn format(&mut self) {
        self.format_with(Formatter::default());
    }

    /// Formats an object and returns the modified value.
    fn formatted_with(mut self, formatter: Formatter) -> Self
    where
        Self: Sized,
    {
        self.format_with(formatter);
        self
    }

    /// Applies the default format to an object and returns the modified value.
    fn formatted(mut self) -> Self
    where
        Self: Sized,
    {
        self.format();
        self
    }
}

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

    fn visit_decor<P, S>(&mut self, decor: &mut Decor, modify_prefix: P, modify_suffix: S)
    where
        P: FnOnce(DecorFormatter) -> DecorFormatter,
        S: FnOnce(DecorFormatter) -> DecorFormatter,
    {
        modify_prefix(decor.prefix.modify()).format(self);
        modify_suffix(decor.suffix.modify()).format(self);
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
