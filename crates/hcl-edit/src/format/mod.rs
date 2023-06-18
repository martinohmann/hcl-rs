//! Format HCL language items.

mod decor;
#[cfg(test)]
mod tests;
mod visit;

use self::decor::{DecorFormatter, ModifyDecor};
use crate::{Decor, Decorate};
use hcl_primitives::InternalString;
use std::ops;

/// A trait for objects which can be formatted.
pub trait Format {
    /// Formats an object.
    fn format(&mut self, config: &FormatConfig);

    /// Applies the default format to an object.
    fn default_format(&mut self) {
        let config = FormatConfig::default();
        self.format(&config);
    }

    /// Formats an object and returns the modified value.
    fn formatted(mut self, config: &FormatConfig) -> Self
    where
        Self: Sized,
    {
        self.format(config);
        self
    }

    /// Applies the default format to an object and returns the modified value.
    fn default_formatted(mut self) -> Self
    where
        Self: Sized,
    {
        self.default_format();
        self
    }
}

/// Configures the behaviour of the [`Format`] trait.
#[derive(Default, Clone, Debug)]
pub struct FormatConfig {
    indent: Indent,
}

impl FormatConfig {
    /// Creates a builder for [`FormatConfig`].
    pub fn builder() -> FormatConfigBuilder {
        FormatConfigBuilder::default()
    }
}

/// A builder for [`FormatConfig`].
#[derive(Default, Clone, Debug)]
pub struct FormatConfigBuilder {
    indent: Indent,
}

impl FormatConfigBuilder {
    /// Sets the indent.
    pub fn indent(&mut self, prefix: impl Into<InternalString>) -> &mut Self {
        self.indent.prefix = prefix.into();
        self
    }

    /// Sets the initial indentation level.
    pub fn initial_indent_level(&mut self, level: usize) -> &mut Self {
        self.indent.level = level;
        self
    }

    /// Builds a [`FormatConfig`] from the builder's configuration.
    pub fn build(&self) -> FormatConfig {
        FormatConfig {
            indent: self.indent.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Formatter {
    indent: Indent,
}

impl Formatter {
    pub fn new(config: &FormatConfig) -> Formatter {
        Formatter {
            indent: config.indent.clone(),
        }
    }
}

/// Applies indentation.
#[derive(Debug, Clone)]
struct Indent {
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
