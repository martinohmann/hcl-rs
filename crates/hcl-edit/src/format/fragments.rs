use crate::util::dedent;
use std::borrow::Cow;
use std::ops;
use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{multispace1, not_line_ending, space1},
    multi::many0,
    Parser,
};

#[derive(Debug)]
pub(crate) enum DecorKind {
    Inline,
    Multiline,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CollapseMode {
    None,
    Start,
    End,
    Both,
}

#[derive(Debug)]
pub(crate) enum DecorFragment<'i> {
    Whitespace(&'i str),
    InlineComment(&'i str),
    HashLineComment(&'i str),
    DoubleSlashLineComment(&'i str),
}

#[derive(Debug)]
pub(crate) struct DecorFragments<'i>(Vec<DecorFragment<'i>>);

impl<'i> DecorFragment<'i> {
    fn as_str(&self) -> &str {
        match self {
            DecorFragment::Whitespace(s) => s,
            DecorFragment::InlineComment(s)
            | DecorFragment::HashLineComment(s)
            | DecorFragment::DoubleSlashLineComment(s) => s,
        }
    }

    fn into_cow_str(self) -> Cow<'i, str> {
        match self {
            DecorFragment::Whitespace(s)
            | DecorFragment::InlineComment(s)
            | DecorFragment::HashLineComment(s)
            | DecorFragment::DoubleSlashLineComment(s) => Cow::Borrowed(s),
        }
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
        }
    }
}

impl<'i> DecorFragments<'i> {
    fn parse(input: &str, kind: DecorKind) -> Option<DecorFragments> {
        match kind {
            DecorKind::Inline => parse_inline(input),
            DecorKind::Multiline => parse_multiline(input),
        }
    }

    fn into_cow_str(mut self) -> Option<Cow<'i, str>> {
        match self.len() {
            0 => None,
            1 => Some(self.remove(0).into_cow_str()),
            _ => Some(Cow::Owned(self.iter().map(DecorFragment::as_str).collect())),
        }
    }

    fn indent(&self, prefix: &str, mut skip_first_line: bool) -> Cow<'i, str> {
        let mut result = Cow::Borrowed("");

        for fragment in &self.0 {
            let indented = fragment.indent(prefix, skip_first_line);
            skip_first_line = !indented.ends_with('\n');
            result.to_mut().push_str(&indented);
        }

        result
    }

    fn collapse_spaces(&mut self, mode: CollapseMode) {
        if matches!(mode, CollapseMode::Start | CollapseMode::Both) {
            if let Some(DecorFragment::Whitespace(s)) = self.first_mut() {
                *s = " ";
            }
        }

        if matches!(mode, CollapseMode::End | CollapseMode::Both) {
            if let Some(DecorFragment::Whitespace(s)) = self.last_mut() {
                *s = " ";
            }
        }
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

fn reindent<'a, S>(s: S, prefix: &str, skip_first: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let dedented = dedent(s, skip_first);
    indent_with(dedented, prefix, skip_first)
}

fn indent_with<'a, S>(s: S, prefix: &str, skip_first: bool) -> Cow<'a, str>
where
    S: Into<Cow<'a, str>>,
{
    let s = s.into();

    if s.is_empty() {
        return Cow::Owned(prefix.to_owned());
    }

    let length = s.len();
    let mut output = String::with_capacity(length + length / 2);

    for (i, line) in s.lines().enumerate() {
        if i > 0 {
            output.push('\n');
            output.push_str(prefix);
        } else if !skip_first {
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
            assert_eq!(indented, expected);
        }
    }
}
