//! Format data structures as HCL.
//!
//! This module provides the [`Formatter`] type and the convienince functions [`to_string`],
//! [`to_vec`] and [`to_writer`] for formatting the data structures provided by this crate as HCL.
//!
//! For serialization of other Rust data structures implementing [`serde::Serialize`] refer to the
//! documentation of the [`ser`](crate::ser) module.
//!
//! # Examples
//!
//! Format an HCL block as string:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let block = hcl::Block::builder("user")
//!     .add_label("johndoe")
//!     .add_attribute(("age", 34))
//!     .add_attribute(("email", "johndoe@example.com"))
//!     .build();
//!
//! let expected = r#"
//! user "johndoe" {
//!   age = 34
//!   email = "johndoe@example.com"
//! }
//! "#.trim_start();
//!
//! let formatted = hcl::format::to_string(&block)?;
//!
//! assert_eq!(formatted, expected);
//! #   Ok(())
//! # }
//! ```

pub(crate) mod escape;
mod impls;

use crate::Result;
use std::io;

mod private {
    pub trait Sealed {}
}

/// A trait to format data structures as HCL.
///
/// This trait is sealed to prevent implementation outside of this crate.
pub trait Format: private::Sealed {
    /// Formats a HCL structure using a formatter and writes the result to the provided writer.
    ///
    /// # Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `Error`.
    fn format<W>(&self, fmt: &mut Formatter<W>) -> Result<()>
    where
        W: io::Write;

    /// Formats a HCL structure using a formatter and returns the result as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `Error`.
    fn format_vec<W>(&self, fmt: &mut Formatter<W>) -> Result<Vec<u8>>
    where
        W: io::Write + AsMut<Vec<u8>>,
    {
        self.format(fmt)?;
        // "Drain" the buffer by splitting off all bytes, leaving the formatter's buffer empty
        // ready for reuse.
        Ok(fmt.writer.as_mut().split_off(0))
    }

    /// Formats a HCL structure using a formatter and returns the result as a `String`.
    ///
    /// # Errors
    ///
    /// Formatting the data structure or writing to the writer may fail with an `Error`.
    fn format_string<W>(&self, fmt: &mut Formatter<W>) -> Result<String>
    where
        W: io::Write + AsMut<Vec<u8>>,
    {
        let bytes = self.format_vec(fmt)?;
        // SAFETY: The `Formatter` never emits invalid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(bytes) })
    }
}

#[derive(PartialEq)]
enum FormatState {
    Initial,
    AttributeStart,
    AttributeEnd,
    BlockStart,
    BlockEnd,
    BlockBodyStart,
}

struct FormatConfig<'a> {
    indent: &'a [u8],
    dense: bool,
    compact_arrays: bool,
    compact_objects: bool,
    prefer_ident_keys: bool,
}

impl<'a> Default for FormatConfig<'a> {
    fn default() -> Self {
        FormatConfig {
            indent: b"  ",
            dense: false,
            compact_arrays: false,
            compact_objects: false,
            prefer_ident_keys: false,
        }
    }
}

/// A pretty printing HCL formatter.
///
/// # Examples
///
/// Format an HCL block as string:
///
/// ```
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use hcl::format::{Format, Formatter};
///
/// let mut buf = Vec::new();
/// let mut formatter = Formatter::new(&mut buf);
///
/// let block = hcl::Block::builder("user")
///     .add_label("johndoe")
///     .add_attribute(("age", 34))
///     .add_attribute(("email", "johndoe@example.com"))
///     .build();
///
/// block.format(&mut formatter)?;
///
/// let expected = r#"
/// user "johndoe" {
///   age = 34
///   email = "johndoe@example.com"
/// }
/// "#.trim_start();
///
/// let formatted = String::from_utf8(buf)?;
///
/// assert_eq!(formatted, expected);
/// #   Ok(())
/// # }
/// ```
///
/// The [`builder()`](Formatter::builder) method can be used to construct a custom `Formatter` for
/// use with a [`Serializer`][Serializer]:
///
/// ```
/// use hcl::{format::Formatter, ser::Serializer};
/// # let mut writer = Vec::new();
///
/// let formatter = Formatter::builder()
///     .indent(b"  ")
///     .dense(false)
///     .build(&mut writer);
///
/// let ser = Serializer::with_formatter(formatter);
/// ```
///
/// [Serializer]: ../ser/struct.Serializer.html
pub struct Formatter<'a, W> {
    writer: W,
    config: FormatConfig<'a>,
    state: FormatState,
    first_element: bool,
    current_indent: usize,
    has_value: bool,
    compact_mode_level: u64,
}

/// A builder to create a `Formatter`.
///
/// See the documentation of [`Formatter`] for a usage example.
pub struct FormatterBuilder<'a> {
    config: FormatConfig<'a>,
}

impl<'a> FormatterBuilder<'a> {
    /// Set the indent for indenting nested HCL structures.
    ///
    /// The default indentation is two spaces.
    pub fn indent(mut self, indent: &'a [u8]) -> Self {
        self.config.indent = indent;
        self
    }

    /// If set, blocks are not visually separated by empty lines from attributes and adjacent
    /// blocks.
    ///
    /// Default formatting:
    ///
    /// ```hcl
    /// attr1 = "value1"
    /// attr2 = "value2"
    ///
    /// block1 {}
    ///
    /// block2 {}
    /// ```
    ///
    /// Dense formatting:
    ///
    /// ```hcl
    /// attr1 = "value1"
    /// attr2 = "value2"
    /// block1 {}
    /// block2 {}
    /// ```
    pub fn dense(mut self, yes: bool) -> Self {
        self.config.dense = yes;
        self
    }

    /// If set, arrays and objects are formatted in a more compact way.
    ///
    /// See the method documation of [`compact_arrays`][FormatterBuilder::compact_arrays] and
    /// [`compact_objects`][FormatterBuilder::compact_objects].
    pub fn compact(self, yes: bool) -> Self {
        self.compact_arrays(yes).compact_objects(yes)
    }

    /// Controls the array formatting.
    ///
    /// By default, array elements are separated by newlines:
    ///
    /// ```hcl
    /// array = [
    ///   1,
    ///   2,
    ///   3,
    /// ]
    /// ```
    ///
    /// When compact array formatting is enabled no newlines are inserted between elements:
    ///
    /// ```hcl
    /// array = [1, 2, 3]
    /// ```
    pub fn compact_arrays(mut self, yes: bool) -> Self {
        self.config.compact_arrays = yes;
        self
    }

    /// Controls the object formatting.
    ///
    /// By default, object items are separated by newlines:
    ///
    /// ```hcl
    /// object = {
    ///   one = "foo"
    ///   two = "bar"
    ///   three = "baz"
    /// }
    /// ```
    ///
    /// When compact object formatting is enabled no newlines are inserted between items:
    ///
    /// ```hcl
    /// object = { one = "foo", two = "bar", three = "baz" }
    /// ```
    pub fn compact_objects(mut self, yes: bool) -> Self {
        self.config.compact_objects = yes;
        self
    }

    /// Controls the object key quoting.
    ///
    /// By default, object keys are formatted as quoted strings (unless they are of variant
    /// [`ObjectKey::Identifier`][ident-variant]).
    ///
    /// ```hcl
    /// object = {
    ///   "foo" = 1
    ///   "bar baz" = 2
    /// }
    /// ```
    ///
    /// When identifier keys are preferred, object keys that are also valid HCL identifiers are
    /// not quoted:
    ///
    /// ```hcl
    /// object = {
    ///   foo = 1
    ///   "bar baz" = 2
    /// }
    /// ```
    ///
    /// [ident-variant]: crate::expr::ObjectKey::Identifier
    pub fn prefer_ident_keys(mut self, yes: bool) -> Self {
        self.config.prefer_ident_keys = yes;
        self
    }

    /// Consumes the `FormatterBuilder` and turns it into a `Formatter` which writes HCL to the
    /// provided writer.
    pub fn build<W>(self, writer: W) -> Formatter<'a, W>
    where
        W: io::Write,
    {
        Formatter {
            writer,
            config: self.config,
            state: FormatState::Initial,
            first_element: false,
            current_indent: 0,
            has_value: false,
            compact_mode_level: 0,
        }
    }

    /// Consumes the `FormatterBuilder` and turns it into a `Formatter` which is specialized to use
    /// a pre-allocated `Vec<u8>` as internal buffer.
    ///
    /// The returned formatter can be passed to the [`format_string`][Format::format_string] or
    /// [`format_vec`][Format::format_vec] method of types implementing [`Format`].
    ///
    /// Alternatively, the internal buffer can be obtained by calling
    /// [`into_inner`][Formatter::into_inner] on the returned `Formatter` after passing it to the
    /// [`format`][Format::format] method of a type implementing [`Format`].
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hcl::format::{Format, Formatter};
    /// use hcl::structure::Attribute;
    ///
    /// let mut formatter = Formatter::builder()
    ///     .compact_arrays(true)
    ///     .build_vec();
    ///
    /// let attr = Attribute::new("foo", vec![1, 2, 3]);
    ///
    /// assert_eq!(attr.format_string(&mut formatter)?, "foo = [1, 2, 3]\n");
    /// #    Ok(())
    /// # }
    /// ```
    pub fn build_vec(self) -> Formatter<'a, Vec<u8>> {
        let vec = Vec::with_capacity(128);
        self.build(vec)
    }
}

impl<'a> Default for Formatter<'a, Vec<u8>> {
    /// Creates the default `Formatter` which is specialized to use a pre-allocated `Vec<u8>` as
    /// internal buffer.
    ///
    /// The formatter can be passed to the [`format_string`][Format::format_string] or
    /// [`format_vec`][Format::format_vec] method of types implementing [`Format`].
    ///
    /// Alternatively, the internal buffer can be obtained by calling
    /// [`into_inner`][Formatter::into_inner] after passing it to the [`format`][Format::format]
    /// method of a type implementing [`Format`].
    fn default() -> Self {
        Formatter::builder().build_vec()
    }
}

// Public API.
impl<'a> Formatter<'a, ()> {
    /// Creates a new [`FormatterBuilder`] to start building a new `Formatter`.
    pub fn builder() -> FormatterBuilder<'a> {
        FormatterBuilder {
            config: FormatConfig::default(),
        }
    }
}

// Public API.
impl<'a, W> Formatter<'a, W>
where
    W: io::Write,
{
    /// Creates a new `Formatter` which writes HCL to the provided writer.
    pub fn new(writer: W) -> Formatter<'a, W> {
        Formatter::builder().build(writer)
    }

    /// Takes ownership of the `Formatter` and returns the underlying writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

// Internal formatter API.
impl<'a, W> Formatter<'a, W>
where
    W: io::Write,
{
    /// Writes `null` to the writer.
    fn write_null(&mut self) -> Result<()> {
        self.write_bytes(b"null")
    }

    /// Writes a boolean value to the writer.
    fn write_bool(&mut self, value: bool) -> Result<()> {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        self.write_bytes(s)
    }

    /// Writes an integer value to the writer.
    fn write_int<T>(&mut self, value: T) -> Result<()>
    where
        T: itoa::Integer,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write_bytes(s.as_bytes())
    }

    /// Writes a quoted string to the writer. The quoted string will be escaped if `escape` is
    /// true.
    fn write_quoted_string(&mut self, s: &str, escape: bool) -> Result<()> {
        self.write_bytes(b"\"")?;
        if escape {
            self.write_escaped_string(s)?;
        } else {
            self.write_string_fragment(s)?;
        }
        self.write_bytes(b"\"")
    }

    /// Writes a string fragment to the writer. No escaping occurs.
    fn write_string_fragment(&mut self, s: &str) -> Result<()> {
        self.write_bytes(s.as_bytes())
    }

    /// Writes a string to the writer and escapes control characters and quotes that might be
    /// contained in it.
    fn write_escaped_string(&mut self, value: &str) -> Result<()> {
        escape::write_escaped_string(&mut self.writer, value)?;
        Ok(())
    }

    /// Signals the start of an array to the formatter.
    fn begin_array(&mut self) -> Result<()> {
        if !self.compact_arrays() {
            self.current_indent += 1;
        }
        self.has_value = false;
        self.first_element = true;
        self.write_bytes(b"[")
    }

    /// Signals the start of an array value to the formatter.
    fn begin_array_value(&mut self) -> Result<()> {
        if self.first_element {
            self.first_element = false;
            if !self.compact_arrays() {
                self.write_bytes(b"\n")?;
                self.write_indent(self.current_indent)?;
            }
        } else if self.compact_arrays() {
            self.write_bytes(b", ")?;
        } else {
            self.write_bytes(b",\n")?;
            self.write_indent(self.current_indent)?;
        }

        Ok(())
    }

    /// Signals the end of an array value to the formatter.
    fn end_array_value(&mut self) -> Result<()> {
        self.has_value = true;
        Ok(())
    }

    /// Signals the end of an array to the formatter.
    fn end_array(&mut self) -> Result<()> {
        if !self.compact_arrays() {
            self.current_indent -= 1;

            if self.has_value {
                self.write_bytes(b"\n")?;
                self.write_indent(self.current_indent)?;
            }
        }

        self.write_bytes(b"]")
    }

    /// Signals the start of an object to the formatter.
    fn begin_object(&mut self) -> Result<()> {
        if !self.compact_objects() {
            self.current_indent += 1;
        }
        self.has_value = false;
        self.first_element = true;
        self.write_bytes(b"{")
    }

    /// Signals the start of an object key to the formatter.
    fn begin_object_key(&mut self) -> Result<()> {
        if self.first_element {
            self.first_element = false;
            if self.compact_objects() {
                self.write_bytes(b" ")?;
            } else {
                self.write_bytes(b"\n")?;
                self.write_indent(self.current_indent)?;
            }
        } else if self.compact_objects() {
            self.write_bytes(b", ")?;
        } else {
            self.write_bytes(b"\n")?;
            self.write_indent(self.current_indent)?;
        }

        Ok(())
    }

    /// Signals the start of an object value to the formatter.
    fn begin_object_value(&mut self) -> Result<()> {
        self.write_bytes(b" = ")
    }

    /// Signals the end of an object value to the formatter.
    fn end_object_value(&mut self) -> Result<()> {
        self.end_array_value()
    }

    /// Signals the end of an object to the formatter.
    fn end_object(&mut self) -> Result<()> {
        if self.compact_objects() {
            if self.has_value {
                self.write_bytes(b" ")?;
            }
        } else {
            self.current_indent -= 1;

            if self.has_value {
                self.write_bytes(b"\n")?;
                self.write_indent(self.current_indent)?;
            }
        }

        self.write_bytes(b"}")
    }

    /// Signals the start of an attribute to the formatter.
    fn begin_attribute(&mut self) -> Result<()> {
        self.maybe_write_newline(FormatState::AttributeStart)?;
        self.write_indent(self.current_indent)
    }

    /// Signals the start of an attribute value to the formatter.
    fn begin_attribute_value(&mut self) -> Result<()> {
        self.write_bytes(b" = ")
    }

    /// Signals the end of an attribute to the formatter.
    fn end_attribute(&mut self) -> Result<()> {
        self.state = FormatState::AttributeEnd;
        self.write_bytes(b"\n")
    }

    /// Signals the start of a block to the formatter.
    fn begin_block(&mut self) -> Result<()> {
        self.maybe_write_newline(FormatState::BlockStart)?;
        self.write_indent(self.current_indent)
    }

    /// Signals the start of a block body to the formatter.
    fn begin_block_body(&mut self) -> Result<()> {
        self.current_indent += 1;
        self.state = FormatState::BlockBodyStart;
        self.write_bytes(b" {")
    }

    /// Signals the end of a block to the formatter.
    fn end_block(&mut self) -> Result<()> {
        self.state = FormatState::BlockEnd;
        self.current_indent -= 1;
        self.write_indent(self.current_indent)?;
        self.write_bytes(b"}\n")
    }

    // Conditionally writes a newline character depending on the formatter configuration and the
    // current and next state. Updates the state to `next_state`.
    fn maybe_write_newline(&mut self, next_state: FormatState) -> Result<()> {
        let newline = match &self.state {
            FormatState::AttributeEnd if !self.config.dense => {
                matches!(next_state, FormatState::BlockStart)
            }
            FormatState::BlockEnd if !self.config.dense => {
                matches!(
                    next_state,
                    FormatState::BlockStart | FormatState::AttributeStart
                )
            }
            other => matches!(other, FormatState::BlockBodyStart),
        };

        if newline {
            self.write_bytes(b"\n")?;
        }

        self.state = next_state;
        Ok(())
    }

    fn write_indent(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            self.write_bytes(self.config.indent)?;
        }

        Ok(())
    }

    fn write_indented(&mut self, n: usize, s: &str) -> Result<()> {
        for (i, line) in s.lines().enumerate() {
            if i > 0 {
                self.write_bytes(b"\n")?;
            }

            self.write_indent(n)?;
            self.write_string_fragment(line)?;
        }

        if s.ends_with('\n') {
            self.write_bytes(b"\n")?;
        }

        Ok(())
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf)?;
        Ok(())
    }

    /// Enables compact mode, runs the closure and disables compact mode again unless it's enabled
    /// via another call to `with_compact_mode`.
    ///
    /// This is mostly used for serializing array and object function arguments.
    fn with_compact_mode<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        self.compact_mode_level += 1;
        let result = f(self);
        self.compact_mode_level -= 1;
        result
    }

    fn compact_arrays(&self) -> bool {
        self.config.compact_arrays || self.in_compact_mode()
    }

    fn compact_objects(&self) -> bool {
        self.config.compact_objects || self.in_compact_mode()
    }

    fn in_compact_mode(&self) -> bool {
        self.compact_mode_level > 0
    }
}

/// Format the given value as an HCL byte vector.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_vec`](crate::to_vec) instead.
///
/// # Errors
///
/// Formatting a value as byte vector cannot fail.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Format,
{
    let mut formatter = Formatter::default();
    value.format_vec(&mut formatter)
}

/// Format the given value as an HCL string.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_string`](crate::to_string) instead.
///
/// # Errors
///
/// Formatting a value as string cannot fail.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Format,
{
    let mut formatter = Formatter::default();
    value.format_string(&mut formatter)
}

/// Format the given value as HCL into the IO stream.
///
/// If you need to serialize custom data structures implementing [`serde::Serialize`] use
/// [`hcl::to_writer`](crate::to_writer) instead.
///
/// # Errors
///
/// Formatting fails if any operation on the writer fails.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Format,
{
    let mut formatter = Formatter::new(writer);
    value.format(&mut formatter)
}

/// Format the given value as an interpolated HCL string.
///
/// It is the callers responsiblity to ensure that the value is not an HCL structure (i.e. `Body`,
/// `Structure`, `Block` or `Attribute`). Otherwise this will produce invalid HCL.
///
/// # Errors
///
/// Formatting a value as string cannot fail.
pub(crate) fn to_interpolated_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Format,
{
    let mut formatter = Formatter::builder().compact(true).build_vec();
    formatter.writer.extend([b'$', b'{']);
    let mut string = value.format_string(&mut formatter)?;
    string.push('}');
    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::to_interpolated_string;
    use crate::expr::{BinaryOp, BinaryOperator, FuncCall};
    use pretty_assertions::assert_eq;

    #[test]
    fn format_interpolated_string() {
        let binop = BinaryOp::new(1, BinaryOperator::Plus, 1);
        assert_eq!(to_interpolated_string(&binop).unwrap(), "${1 + 1}");

        let expr = FuncCall::builder("add").arg(1).arg(1).build();
        assert_eq!(to_interpolated_string(&expr).unwrap(), "${add(1, 1)}");
    }
}
