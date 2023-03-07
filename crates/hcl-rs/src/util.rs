use crate::{Error, Result};
use std::borrow::Cow;
use std::str::Chars;

// String dedent implementation which does not distinguish between spaces, tabs or unicode
// whitespace but simply treats all of them as "one unit of whitespace".
//
// This is how the original HCL spec seems to handle it based on the original specsuite although it
// is not formally defined. E.g. ' ' (space) and '\u{2003}' (unicode "em-space") are treated as one
// unit of whitespace even though the former is 1 byte and the latter is 3 bytes long.
#[cfg(feature = "pest")]
pub(crate) fn dedent(s: &str, skip_first: bool) -> Cow<str> {
    let min_leading_ws = min_leading_whitespace(s, skip_first);

    if min_leading_ws == 0 {
        return Cow::Borrowed(s);
    }

    dedent_by(s, min_leading_ws, skip_first)
}

pub(crate) fn dedent_by(s: &str, n: usize, skip_first: bool) -> Cow<str> {
    let mut dedented = String::with_capacity(s.len());

    for (i, line) in s.lines().enumerate() {
        if !(line.is_empty() || i == 0 && skip_first) {
            dedented.extend(line.chars().skip(n));
        }

        dedented.push('\n');
    }

    if dedented.ends_with('\n') && !s.ends_with('\n') {
        let new_len = dedented.len() - 1;
        dedented.truncate(new_len);
    }

    dedented.shrink_to_fit();

    Cow::Owned(dedented)
}

pub(crate) fn min_leading_whitespace(s: &str, skip_first: bool) -> usize {
    if s.is_empty() {
        return 0;
    }

    let mut leading_ws = usize::MAX;

    // Find the minimum number of possible leading units of whitespace that can be be stripped off
    // of each non-empty line.
    for (i, line) in s.lines().enumerate() {
        if (i == 0 && skip_first) || line.is_empty() {
            continue;
        }

        let line_leading_ws = line.chars().take_while(|ch| ch.is_whitespace()).count();

        if line_leading_ws == 0 {
            // Fast path: no dedent needed if we encounter a non-empty line which starts with a
            // non-whitespace character.
            return 0;
        }

        leading_ws = leading_ws.min(line_leading_ws);
    }

    leading_ws
}

#[cfg(feature = "winnow")]
pub(crate) fn indent_by(s: &str, n: usize, skip_first: bool) -> String {
    let prefix = " ".repeat(n);
    let length = s.len();
    let mut output = String::with_capacity(length + length / 2);

    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            output.push('\n');

            if !line.is_empty() {
                output.push_str(&prefix);
            }
        } else if !skip_first && !line.is_empty() {
            output.push_str(&prefix);
        }

        output.push_str(line);
    }

    if s.ends_with('\n') {
        output.push('\n');
    }

    output
}

/// Takes in a string with backslash escapes written out with literal backslash characters and
/// converts it to a string with the proper escaped characters.
///
/// ## Errors
///
/// Returns an error if an invalid or incomplete escape sequence or unicode code point is
/// encountered.
pub fn unescape(s: &str) -> Result<Cow<str>> {
    for (idx, ch) in s.chars().enumerate() {
        if ch == '\\' {
            // At least one char needs unescaping so we need to return a new `String` instead of a
            // borrowed `&str`.
            return unescape_owned(s, idx).map(Cow::Owned);
        }
    }

    Ok(Cow::Borrowed(s))
}

fn unescape_owned(s: &str, idx: usize) -> Result<String> {
    let mut buf = String::with_capacity(s.len());

    // Put all preceeding chars into buf already.
    buf.push_str(&s[..idx]);

    let mut chars = s[idx..].chars();
    let mut scratch = String::new();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            buf.push(ch);
            continue;
        }

        let ch = match chars.next() {
            Some('b') => '\u{0008}',
            Some('f') => '\u{000C}',
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('\'') => '\'',
            Some('\"') => '\"',
            Some('\\') => '\\',
            Some('u') => match unescape_unicode(&mut chars, &mut scratch) {
                Some(ch) => ch,
                None => return Err(Error::InvalidUnicodeCodePoint(scratch)),
            },
            Some(ch) => return Err(Error::InvalidEscape(ch)),
            None => return Err(Error::Eof),
        };

        buf.push(ch);
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

/// Like [`unescape`], but returns the original `&str` if it contains invalid escape sequences
/// instead of failing.
pub fn try_unescape(s: &str) -> Cow<str> {
    match unescape(s) {
        Ok(s) => s,
        Err(_) => Cow::Borrowed(s),
    }
}

/// Scan `s` for sequences that introduce a template interpolation or directive. Returns `true`
/// once it found one of these start markers, `false` otherwise.
///
/// This function only looks for start markers and does not check if the template is actually
/// valid.
#[inline]
pub fn is_templated(s: &str) -> bool {
    if s.len() < 3 {
        return false;
    }

    let mut skip_next = false;

    // Because calling `s.contains("${")` would also match escaped interpolations (`$${`) a
    // window iterator is used here to detect and ignore these. The same applies to escaped
    // directives.
    for window in s.as_bytes().windows(3) {
        if skip_next {
            skip_next = false;
            continue;
        }

        match window {
            [b'$', b'$', b'{'] | [b'%', b'%', b'{'] => {
                // The next window would incorrectly match the next arm, so it must be
                // skipped.
                skip_next = true;
            }
            [b'$' | b'%', b'{', _] => return true,
            _ => {}
        }
    }

    false
}

/// Determines if `ch` is a valid HCL identifier start character.
#[inline]
pub fn is_id_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}

/// Determines if `ch` is a valid HCL identifier continue character.
#[inline]
pub fn is_id_continue(ch: char) -> bool {
    ch == '-' || unicode_ident::is_xid_continue(ch)
}

/// Determines if `s` represents a valid HCL identifier.
#[inline]
pub fn is_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    is_id_start(first) && chars.all(is_id_continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_templated() {
        assert!(is_templated("${a}"));
        assert!(is_templated("${\"a\"}"));
        assert!(is_templated("%{ if foo }foo%{ else }bar%{ endif }"));
        assert!(is_templated("$${ introduces an ${\"interpolation\"}"));
        assert!(!is_templated(
            "escaped directive %%{ if foo }foo%%{ else }bar%%{ endif }"
        ));
        assert!(!is_templated("escaped interpolation $${a}"));
    }
}
