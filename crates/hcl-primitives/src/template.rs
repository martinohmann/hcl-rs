//! Primitives for the HCL template sub-language.

use alloc::borrow::Cow;

/// Controls the whitespace strip behaviour for template interpolations and directives on adjacent
/// string literals.
///
/// The strip behaviour is controlled by a `~` immediately following an interpolation (`${`) or
/// directive (`%{`) introduction, or preceding the closing `}`.
///
/// Whitespace is stripped up until (and including) the next line break:
///
/// - `${~ expr}` strips whitespace from an immediately **preceding** string literal.
/// - `${expr ~}` strips whitespace from an immediately **following** string literal.
/// - `${~ expr ~}` strips whitespace from immediately **preceding** and **following** string
///   literals.
/// - `${expr}` does not strip any whitespace.
///
/// The stripping behaviour is equivalent for template directives (`%{expr}`).
///
/// For more details, check the section about template literals in the [HCL syntax
/// specification](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#template-literals).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Strip {
    /// Don't strip adjacent spaces.
    #[default]
    None,
    /// Strip any adjacent spaces from the immediately preceding string literal, if there is
    /// one.
    Start,
    /// Strip any adjacent spaces from the immediately following string literal, if there is one.
    End,
    /// Strip any adjacent spaces from the immediately preceding and following string literals,
    /// if there are any.
    Both,
}

impl Strip {
    /// Returns `true` if adjacent spaces should be stripped from an immediately preceding string
    /// literal.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl_primitives::template::Strip;
    /// assert!(!Strip::None.strip_start());
    /// assert!(Strip::Start.strip_start());
    /// assert!(!Strip::End.strip_start());
    /// assert!(Strip::Both.strip_start());
    /// ```
    pub fn strip_start(self) -> bool {
        matches!(self, Strip::Start | Strip::Both)
    }

    /// Returns `true` if adjacent spaces should be stripped from an immediately following string
    /// literal.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl_primitives::template::Strip;
    /// assert!(!Strip::None.strip_end());
    /// assert!(!Strip::Start.strip_end());
    /// assert!(Strip::End.strip_end());
    /// assert!(Strip::Both.strip_end());
    /// ```
    pub fn strip_end(self) -> bool {
        matches!(self, Strip::End | Strip::Both)
    }
}

impl From<(bool, bool)> for Strip {
    fn from((start, end): (bool, bool)) -> Self {
        match (start, end) {
            (true, true) => Strip::Both,
            (true, false) => Strip::Start,
            (false, true) => Strip::End,
            (false, false) => Strip::None,
        }
    }
}

/// Escapes interpolation sequence (`${`) and directive control flow (`%{`) start markers in a
/// string literal to `$${` and `%%{` respectively.
///
/// ```
/// use hcl_primitives::template::escape_markers;
///
/// assert_eq!(escape_markers("foo"), "foo");
/// assert_eq!(escape_markers("${interpolation}"), "$${interpolation}");
/// assert_eq!(escape_markers("$${escaped_interpolation}"), "$$${escaped_interpolation}");
/// assert_eq!(escape_markers("%{if foo}bar%{else}baz%{endif}"), "%%{if foo}bar%%{else}baz%%{endif}");
/// ```
pub fn escape_markers(literal: &str) -> Cow<str> {
    if literal.len() < 2 {
        // Fast path: strings shorter than 2 chars cannot contain `${` or `%{`.
        return Cow::Borrowed(literal);
    }

    for (idx, window) in literal.as_bytes().windows(2).enumerate() {
        if let b"${" | b"%{" = window {
            // Found start marker, enter slow path.
            return Cow::Owned(escape_markers_owned(literal, idx));
        }
    }

    Cow::Borrowed(literal)
}

fn escape_markers_owned(literal: &str, idx: usize) -> String {
    let (mut buf, rest) = split_buf(literal, idx);
    let mut chars = rest.chars();

    while let Some(ch) = chars.next() {
        buf.push(ch);

        if ch != '$' && ch != '%' {
            continue;
        }

        match chars.next() {
            Some(ch2) => {
                if ch2 == '{' {
                    // Escape the start marker by doubling `ch`.
                    buf.push(ch);
                }

                buf.push(ch2);
            }
            None => break,
        }
    }

    buf
}

/// Unescapes escaped interpolation sequence (`$${`) and directive control flow (`%%{`) start
/// markers in a string literal to `${` and `%{` respectively.
///
/// ```
/// use hcl_primitives::template::unescape_markers;
///
/// assert_eq!(unescape_markers("foo"), "foo");
/// assert_eq!(unescape_markers("${interpolation}"), "${interpolation}");
/// assert_eq!(unescape_markers("$${escaped_interpolation}"), "${escaped_interpolation}");
/// assert_eq!(unescape_markers("$$${escaped_interpolation}"), "$${escaped_interpolation}");
/// assert_eq!(unescape_markers("%{if foo}bar%{else}baz%{endif}"), "%{if foo}bar%{else}baz%{endif}");
/// ```
pub fn unescape_markers(literal: &str) -> Cow<str> {
    if literal.len() < 3 {
        // Fast path: strings shorter than 3 chars cannot contain `$${` or `%%{`.
        return Cow::Borrowed(literal);
    }

    for (idx, window) in literal.as_bytes().windows(3).enumerate() {
        if let b"$${" | b"%%{" = window {
            // Found escaped start marker, enter slow path.
            return Cow::Owned(unescape_markers_owned(literal, idx));
        }
    }

    Cow::Borrowed(literal)
}

fn unescape_markers_owned(literal: &str, idx: usize) -> String {
    let (mut buf, rest) = split_buf(literal, idx);
    let mut chars = rest.chars();

    while let Some(ch) = chars.next() {
        buf.push(ch);

        if ch != '$' && ch != '%' {
            continue;
        }

        match (chars.next(), chars.next()) {
            (Some(ch2), Some('{')) if ch2 == ch => {
                // Unescape by not pushing `ch2` to the output buffer.
                buf.push('{');
            }
            (Some(ch2), ch3) => {
                buf.push(ch2);

                if let Some(ch) = ch3 {
                    buf.push(ch);
                }
            }
            (_, _) => break,
        }
    }
    buf
}

fn split_buf(s: &str, idx: usize) -> (String, &str) {
    let mut buf = String::with_capacity(s.len());
    buf.push_str(&s[..idx]);
    (buf, &s[idx..])
}
