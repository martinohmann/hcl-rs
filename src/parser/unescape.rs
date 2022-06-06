use crate::{Error, Result};
use std::str::Chars;

/// Takes in a string with backslash escapes written out with literal backslash characters and
/// converts it to a string with the proper escaped characters.
///
/// ## Errors
///
/// Returns an error if an invalid or incomplete escape sequence or unicode code point is
/// encountered.
pub fn unescape(s: &str) -> Result<String> {
    let mut buf = String::with_capacity(s.len());
    let mut chars = s.chars();
    let mut scratch = String::new();

    while let Some(c) = chars.next() {
        if c != '\\' {
            buf.push(c);
            continue;
        }

        let c = match chars.next() {
            Some('b') => '\u{0008}',
            Some('f') => '\u{000C}',
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('\'') => '\'',
            Some('\"') => '\"',
            Some('\\') => '\\',
            Some('u') => match unescape_unicode(&mut chars, &mut scratch) {
                Some(c) => c,
                None => return Err(Error::InvalidUnicodeCodePoint(scratch)),
            },
            Some(c) => return Err(Error::InvalidEscape(c)),
            None => return Err(Error::Eof),
        };

        buf.push(c);
    }

    Ok(buf)
}

fn unescape_unicode(chars: &mut Chars<'_>, scratch: &mut String) -> Option<char> {
    scratch.clear();

    for _ in 0..4 {
        scratch.push(chars.next()?);
    }

    char::from_u32(u32::from_str_radix(scratch, 16).ok()?)
}
