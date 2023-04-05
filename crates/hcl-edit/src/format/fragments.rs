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
    HashLineComment(&'i str),
    DoubleSlashLineComment(&'i str),
    Newline,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct DecorFragments<'i>(Vec<DecorFragment<'i>>);

impl<'i> DecorFragment<'i> {
    fn is_whitespace(&self) -> bool {
        matches!(self, DecorFragment::Newline | DecorFragment::Whitespace(_))
    }

    fn indent(&self, prefix: &str, skip_first_line: bool) -> Cow<'i, str> {
        match self {
            DecorFragment::Whitespace(s)
            | DecorFragment::DoubleSlashLineComment(s)
            | DecorFragment::HashLineComment(s) => reindent(*s, prefix, skip_first_line),
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
            DecorFragment::Newline => Cow::Owned(format!("\n{}", prefix)),
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

    pub fn indent(&self, prefix: &str, mut skip_first_line: bool) -> RawString {
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

    pub fn format(&self, formatter: &mut Formatter) -> RawString {
        self.indent_with(&mut formatter.indenter)
    }

    pub fn indent_with(&self, indenter: &mut Indenter) -> RawString {
        let indented = self.indent(&indenter.prefix(), indenter.skip_first_line);
        indenter.skip_first_line = !indented.ends_with('\n');
        indented
    }

    pub fn leading_newline(&mut self) -> &mut Self {
        match self.first_mut() {
            Some(first) if first.is_whitespace() => *first = DecorFragment::Newline,
            Some(DecorFragment::Newline) => {}
            _ => self.insert(0, DecorFragment::Newline),
        }
        self
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
    fn parse_decor(&self, kind: DecorKind) -> DecorFragments;

    fn multiline_decor(&self) -> DecorFragments {
        self.parse_decor(DecorKind::Multiline)
    }

    fn inline_decor(&self) -> DecorFragments {
        self.parse_decor(DecorKind::Inline)
    }
}

impl ParseDecor for RawString {
    fn parse_decor(&self, kind: DecorKind) -> DecorFragments {
        DecorFragments::new(self, kind)
    }
}

impl ParseDecor for Option<&RawString> {
    fn parse_decor(&self, kind: DecorKind) -> DecorFragments {
        let raw = self.map(RawString::as_str).unwrap_or_default();
        DecorFragments::new(raw, kind)
    }
}

fn parse_multiline(input: &str) -> Option<DecorFragments> {
    many0::<_, _, _, (), _>(alt((
        multispace1.map(DecorFragment::Whitespace),
        ('#', not_line_ending)
            .recognize()
            .map(DecorFragment::HashLineComment),
        ("//", not_line_ending)
            .recognize()
            .map(DecorFragment::DoubleSlashLineComment),
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
