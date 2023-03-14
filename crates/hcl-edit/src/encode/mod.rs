mod expr;
mod structure;
mod template;

use crate::repr::{Decorate, Decorated, Formatted, ValueRepr};
use crate::{Ident, Number};
use std::fmt::{self, Write};

pub(crate) const NO_DECOR: (&str, &str) = ("", "");
const LEADING_SPACE_DECOR: (&str, &str) = (" ", "");
const TRAILING_SPACE_DECOR: (&str, &str) = ("", " ");
const BOTH_SPACE_DECOR: (&str, &str) = (" ", " ");

pub(crate) trait EncodeDecorated {
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result;
}

pub(crate) trait Encode {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result;
}

pub(crate) struct EncodeState<'a> {
    buf: &'a mut dyn fmt::Write,
    escape: bool,
}

impl<'a> EncodeState<'a> {
    pub fn new(buf: &'a mut dyn fmt::Write) -> EncodeState<'a> {
        EncodeState { buf, escape: false }
    }

    pub fn escaped<F>(&mut self, f: F) -> fmt::Result
    where
        F: FnOnce(&mut EncodeState) -> fmt::Result,
    {
        self.escape = true;
        let result = f(self);
        self.escape = false;
        result
    }

    pub fn escape(&self) -> bool {
        self.escape
    }
}

impl<'a> fmt::Write for EncodeState<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buf.write_str(s)
    }
}

impl<T> EncodeDecorated for T
where
    T: Decorate + Encode,
{
    fn encode_decorated(&self, buf: &mut EncodeState, default_decor: (&str, &str)) -> fmt::Result {
        encode_decorated(self, buf, default_decor, |buf| self.encode(buf))
    }
}

impl<T> Encode for Decorated<T>
where
    T: Encode,
{
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        (&**self).encode(buf)
    }
}

impl<T> Encode for Formatted<T>
where
    T: Encode + ValueRepr,
{
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        if let Some(repr) = self.repr() {
            repr.encode_with_default(buf, "")
        } else {
            (&**self).encode(buf)
        }
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{self}")
    }
}

impl Encode for u64 {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{self}")
    }
}

impl Encode for Number {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        write!(buf, "{self}")
    }
}

impl Encode for Ident {
    fn encode(&self, buf: &mut EncodeState) -> fmt::Result {
        buf.write_str(self.as_str())
    }
}

fn encode_decorated<T, F>(
    item: &T,
    buf: &mut EncodeState,
    default_decor: (&str, &str),
    f: F,
) -> fmt::Result
where
    T: ?Sized + Decorate,
    F: FnOnce(&mut EncodeState) -> fmt::Result,
{
    let decor = item.decor();
    decor.encode_prefix(buf, default_decor.0)?;
    f(buf)?;
    decor.encode_suffix(buf, default_decor.1)
}

fn encode_quoted_string(buf: &mut dyn fmt::Write, value: &str) -> fmt::Result {
    buf.write_char('"')?;
    encode_escaped(buf, value)?;
    buf.write_char('"')
}

fn encode_escaped(buf: &mut dyn fmt::Write, value: &str) -> fmt::Result {
    for ch in value.chars() {
        match ch {
            '\u{8}' => buf.write_str("\\b")?,
            '\u{9}' => buf.write_str("\\t")?,
            '\u{a}' => buf.write_str("\\n")?,
            '\u{c}' => buf.write_str("\\f")?,
            '\u{d}' => buf.write_str("\\r")?,
            '\u{22}' => buf.write_str("\\\"")?,
            '\u{5c}' => buf.write_str("\\\\")?,
            c if c <= '\u{1f}' || c == '\u{7f}' => {
                write!(buf, "\\u{:04X}", ch as u32)?;
            }
            ch => buf.write_char(ch)?,
        }
    }

    Ok(())
}
