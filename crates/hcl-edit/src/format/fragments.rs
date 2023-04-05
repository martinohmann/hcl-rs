use super::{Formatter, Indenter};
use crate::{util::dedent, RawString};
use std::borrow::Cow;
use std::ops;
use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{multispace1, not_line_ending, space1},
    multi::many0,
    Parser,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum DecorKind {
    Inline,
    Multiline,
}

#[derive(Debug, Clone)]
pub(crate) enum DecorFragment<'i> {
    Whitespace(&'i str),
    InlineComment(&'i str),
    LineComment(&'i str),
    NewlineIndent,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct DecorFragments<'i>(Vec<DecorFragment<'i>>);

impl<'i> DecorFragment<'i> {
    fn is_inline_comment(&self) -> bool {
        matches!(self, DecorFragment::InlineComment(_))
    }

    fn is_line_comment(&self) -> bool {
        matches!(self, DecorFragment::LineComment(_))
    }

    fn is_comment(&self) -> bool {
        self.is_inline_comment() || self.is_line_comment()
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, DecorFragment::Whitespace(_))
    }

    fn indent(&self, prefix: &str, skip_first_line: bool) -> Cow<'i, str> {
        match self {
            DecorFragment::Whitespace(s) | DecorFragment::LineComment(s) => {
                reindent(*s, prefix, skip_first_line)
            }
            // Since inline comments can contain significant newline characters, we must only
            // indent the first line.
            DecorFragment::InlineComment(s) => {
                let mut lines = s.lines();

                match lines.next() {
                    None => {
                        // Empty inline comment, this cannot be emitted by the parser, but we
                        // handle it anyways.
                        if skip_first_line {
                            Cow::Borrowed("")
                        } else {
                            Cow::Owned(prefix.to_owned())
                        }
                    }
                    Some(first) => {
                        let mut indented = reindent(first, prefix, skip_first_line);

                        // Appends the rest of the inline comment without altering the existing
                        // indent.
                        for line in lines {
                            let res = indented.to_mut();
                            res.push('\n');
                            res.push_str(line);
                        }

                        if s.ends_with('\n') {
                            indented.to_mut().push('\n');
                        }

                        indented
                    }
                }
            }
            DecorFragment::NewlineIndent => Cow::Owned(format!("\n{prefix}")),
        }
    }
}

impl<'i> DecorFragments<'i> {
    pub fn new(input: &str, kind: DecorKind) -> DecorFragments {
        DecorFragments::parse(dbg!(input), kind).unwrap_or_default()
    }

    pub fn parse(input: &str, kind: DecorKind) -> Option<DecorFragments> {
        match kind {
            DecorKind::Inline => parse_inline(input),
            DecorKind::Multiline => parse_multiline(input),
        }
    }

    pub fn format(&self, formatter: &mut Formatter) -> RawString {
        self.indent_with(&mut formatter.indenter)
    }

    pub fn leading_newline(&mut self) -> &mut Self {
        if self.first().map_or(false, DecorFragment::is_whitespace) {
            *self.first_mut().unwrap() = DecorFragment::NewlineIndent;
        } else {
            self.insert(0, DecorFragment::NewlineIndent);
        }

        self
    }

    pub fn indent_empty_trailing_line(&mut self) -> &mut Self {
        if let Some(DecorFragment::Whitespace(s)) = self.last() {
            if let Some(trimmed) = s.trim_end_matches(is_space).strip_suffix('\n') {
                *self.last_mut().unwrap() = DecorFragment::Whitespace(trimmed);
                self.push(DecorFragment::NewlineIndent);
            }
        }

        self
    }

    pub fn trim_trailing_whitespace(&mut self) -> &mut Self {
        if self.last().map_or(false, DecorFragment::is_whitespace) {
            self.pop();
        }

        self
    }

    pub fn space_padded(&mut self) -> &mut Self {
        if self.is_empty() {
            self.push(DecorFragment::Whitespace(" "));
        } else {
            self.space_padded_start().space_padded_end();
        }

        self
    }

    pub fn space_padded_start(&mut self) -> &mut Self {
        if self.first().map_or(false, DecorFragment::is_comment) {
            self.insert(0, DecorFragment::Whitespace(" "));
        }

        self
    }

    pub fn space_padded_end(&mut self) -> &mut Self {
        if self.last().map_or(false, DecorFragment::is_inline_comment) {
            self.push(DecorFragment::Whitespace(" "));
        }

        self
    }

    fn indent(&self, prefix: &str, mut skip_first_line: bool) -> RawString {
        if self.is_empty() && !prefix.is_empty() && !skip_first_line {
            return prefix.into();
        }

        let mut result = Cow::Borrowed("");

        for fragment in self.iter() {
            let indented = fragment.indent(prefix, skip_first_line);
            skip_first_line = !indented.ends_with('\n');
            result.to_mut().push_str(&indented);
        }

        result.into()
    }

    fn indent_with(&self, indenter: &mut Indenter) -> RawString {
        let indented = self.indent(&indenter.prefix(), indenter.skip_first_line);
        indenter.skip_first_line = !indented.ends_with('\n');
        indented
    }
}

impl<'i> ops::Deref for DecorFragments<'i> {
    type Target = Vec<DecorFragment<'i>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'i> ops::DerefMut for DecorFragments<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) trait ParseDecor {
    fn parse_as(&self, kind: DecorKind) -> DecorFragments;

    fn parse_multiline(&self) -> DecorFragments {
        self.parse_as(DecorKind::Multiline)
    }

    fn parse_inline(&self) -> DecorFragments {
        self.parse_as(DecorKind::Inline)
    }
}

impl ParseDecor for RawString {
    fn parse_as(&self, kind: DecorKind) -> DecorFragments {
        DecorFragments::new(self, kind)
    }
}

impl ParseDecor for Option<&RawString> {
    fn parse_as(&self, kind: DecorKind) -> DecorFragments {
        let raw = self.map(RawString::as_str).unwrap_or_default();
        DecorFragments::new(raw, kind)
    }
}

fn parse_multiline(input: &str) -> Option<DecorFragments> {
    many0::<_, _, _, (), _>(alt((
        multispace1.map(DecorFragment::Whitespace),
        ('#', not_line_ending)
            .recognize()
            .map(DecorFragment::LineComment),
        ("//", not_line_ending)
            .recognize()
            .map(DecorFragment::LineComment),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorFragment::InlineComment),
    )))
    .parse(input)
    .map(DecorFragments)
    .ok()
}

fn parse_inline(input: &str) -> Option<DecorFragments> {
    many0::<_, _, _, (), _>(alt((
        space1.map(DecorFragment::Whitespace),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorFragment::InlineComment),
    )))
    .parse(input)
    .map(DecorFragments)
    .ok()
}

fn is_space(ch: char) -> bool {
    ch.is_whitespace() && ch != '\r' && ch != '\n'
}

fn reindent<'a, S>(s: S, prefix: &str, skip_first_line: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let dedented = dedent(s, skip_first_line);
    indent_with(dedented, prefix, skip_first_line)
}

fn indent_with<'a, S>(s: S, prefix: &str, skip_first_line: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let s = s.into();

    if s.is_empty() {
        return Cow::Owned(prefix.to_owned());
    }

    if s == "\n" || s == "\r\n" {
        return Cow::Owned(format!("\n{}", prefix));
    }

    let length = s.len();
    let mut output = String::with_capacity(length + length / 2);

    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            output.push('\n');
            if !line.is_empty() {
                output.push_str(prefix);
            }
        } else if !skip_first_line && !line.is_empty() {
            output.push_str(prefix);
        }

        output.push_str(line);
    }

    if s.ends_with('\n') {
        output.push('\n');
    }

    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_indent() {
        let tests = [
            (
                "  // foo \n/* bar\nbaz */   # 123\n#456",
                "    // foo \n    /* bar\nbaz */   # 123\n    #456",
                DecorKind::Multiline,
            ),
            (
                "  /* bar\nbaz */ \t \t /* qux */",
                "    /* bar\nbaz */ \t \t /* qux */",
                DecorKind::Inline,
            ),
        ];

        for (input, expected, kind) in tests {
            let fragments = DecorFragments::parse(input, kind).unwrap();
            let indented = fragments.indent("    ", false);
            assert_eq!(indented, expected.into());
        }
    }
}
