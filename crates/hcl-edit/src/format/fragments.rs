use super::{Formatter, Indent};
use crate::{util::dedent, RawString};
use std::borrow::Cow;
use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{multispace1, not_line_ending, space1},
    multi::many0,
    Parser,
};

#[derive(Debug, Clone, Copy)]
enum DecorKind {
    Inline,
    Multiline,
}

#[derive(Debug, Clone)]
enum DecorFragment<'a> {
    Whitespace(&'a str),
    InlineComment(&'a str),
    LineComment(&'a str),
    NewlineIndent,
}

#[derive(Debug, Default, Clone)]
struct DecorFragments<'a>(Vec<DecorFragment<'a>>);

impl<'a> DecorFragment<'a> {
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

    fn indent(&self, prefix: &str, skip_first_line: bool) -> Cow<'a, str> {
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

impl<'a> DecorFragments<'a> {
    fn parse(input: &str, kind: DecorKind) -> Option<DecorFragments> {
        let fragments = match kind {
            DecorKind::Inline => parse_inline(input)?,
            DecorKind::Multiline => parse_multiline(input)?,
        };
        Some(DecorFragments(fragments))
    }

    fn leading_newline(&mut self) -> &mut Self {
        if self.0.first().map_or(false, DecorFragment::is_whitespace) {
            *self.0.first_mut().unwrap() = DecorFragment::NewlineIndent;
        } else {
            self.0.insert(0, DecorFragment::NewlineIndent);
        }

        self
    }

    fn indent_empty_trailing_line(&mut self) -> &mut Self {
        if let Some(DecorFragment::Whitespace(s)) = self.0.last() {
            if let Some(trimmed) = s.trim_end_matches(is_space).strip_suffix('\n') {
                *self.0.last_mut().unwrap() = DecorFragment::Whitespace(trimmed);
                self.0.push(DecorFragment::NewlineIndent);
            }
        }

        self
    }

    fn trim(&mut self, trim: Trim) -> &mut Self {
        match trim {
            Trim::None => self,
            Trim::Start => self.trim_start(),
            Trim::End => self.trim_end(),
            Trim::Both => self.trim_start().trim_end(),
        }
    }

    fn trim_start(&mut self) -> &mut Self {
        if self.0.first().map_or(false, DecorFragment::is_whitespace) {
            self.0.remove(0);
        }

        self
    }

    fn trim_end(&mut self) -> &mut Self {
        let len = self.0.len();

        if len > 1 && matches!(&self.0[len - 2], DecorFragment::LineComment(_)) {
            // Whitespace after a line comment contains significant newlines which must not be
            // removed. Only remove spaces.
            if let Some(DecorFragment::Whitespace(s)) = self.0.last_mut() {
                *s = s.trim_matches(is_space);
            }
        } else if self.0.last().map_or(false, DecorFragment::is_whitespace) {
            self.0.pop();
        }

        self
    }

    fn pad(&mut self, padding: Padding) -> &mut Self {
        match padding {
            Padding::None => self,
            Padding::Start => self.pad_start(),
            Padding::End => self.pad_end(),
            Padding::Both => self.pad_both(),
        }
    }

    fn pad_both(&mut self) -> &mut Self {
        if self.0.is_empty() {
            self.0.push(DecorFragment::Whitespace(" "));
            self
        } else {
            self.pad_start().pad_end()
        }
    }

    fn pad_start(&mut self) -> &mut Self {
        if self.0.first().map_or(false, DecorFragment::is_comment) {
            self.0.insert(0, DecorFragment::Whitespace(" "));
        }

        self
    }

    fn pad_end(&mut self) -> &mut Self {
        if self
            .0
            .last()
            .map_or(false, DecorFragment::is_inline_comment)
        {
            self.0.push(DecorFragment::Whitespace(" "));
        }

        self
    }

    fn indent(&self, prefix: &str, mut skip_first_line: bool) -> RawString {
        if self.0.is_empty() && !prefix.is_empty() && !skip_first_line {
            return prefix.into();
        }

        let mut result = Cow::Borrowed("");

        for fragment in self.0.iter() {
            let indented = fragment.indent(prefix, skip_first_line);
            skip_first_line = !indented.ends_with('\n');
            result.to_mut().push_str(&indented);
        }

        result.into()
    }

    fn indent_with(&self, indent: &mut Indent, indent_first_line: Option<bool>) -> RawString {
        let indent_first_line = indent_first_line.unwrap_or(indent.indent_first_line);
        let indented = self.indent(&indent.prefix(), !indent_first_line);
        indent.indent_first_line = indented.ends_with('\n');
        indented
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Padding {
    None,
    Start,
    End,
    Both,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Trim {
    None,
    Start,
    End,
    Both,
}

pub(super) struct DecorFormatter<'a> {
    raw: &'a mut dyn RawStringAccess,
    kind: DecorKind,
    leading_newline: bool,
    indent_empty_trailing_line: bool,
    indent_first_line: Option<bool>,
    trim: Trim,
    padding: Padding,
}

impl<'a> DecorFormatter<'a> {
    fn new(raw: &'a mut dyn RawStringAccess) -> DecorFormatter<'a> {
        DecorFormatter {
            raw,
            kind: DecorKind::Multiline,
            leading_newline: false,
            indent_empty_trailing_line: false,
            indent_first_line: None,
            trim: Trim::None,
            padding: Padding::None,
        }
    }

    pub(super) fn inline(mut self) -> Self {
        self.kind = DecorKind::Inline;
        self
    }

    pub(super) fn leading_newline(mut self) -> Self {
        self.leading_newline = true;
        self
    }

    pub(super) fn indent_empty_trailing_line(mut self) -> Self {
        self.indent_empty_trailing_line = true;
        self
    }

    pub(super) fn indent_first_line(mut self, yes: bool) -> Self {
        self.indent_first_line = Some(yes);
        self
    }

    pub(super) fn trim(mut self, trim: Trim) -> Self {
        self.trim = trim;
        self
    }

    pub(super) fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub(super) fn format(self, formatter: &mut Formatter) {
        let mut fragments = DecorFragments::parse(self.raw.get(), self.kind).unwrap_or_default();

        fragments.trim(self.trim).pad(self.padding);

        if self.leading_newline {
            fragments.leading_newline();
        }

        if self.indent_empty_trailing_line {
            fragments.indent_empty_trailing_line();
        }

        let formatted = fragments.indent_with(&mut formatter.indent, self.indent_first_line);

        self.raw.set(formatted);
    }
}

pub(super) trait RawStringAccess {
    fn get(&self) -> &str;
    fn set(&mut self, raw: RawString);
}

impl RawStringAccess for RawString {
    fn get(&self) -> &str {
        self.as_str()
    }

    fn set(&mut self, raw: RawString) {
        *self = raw;
    }
}

impl RawStringAccess for Option<RawString> {
    fn get(&self) -> &str {
        match self {
            Some(raw) => raw.get(),
            None => "",
        }
    }

    fn set(&mut self, raw: RawString) {
        *self = Some(raw);
    }
}

pub(super) trait ModifyDecor {
    fn modify(&mut self) -> DecorFormatter<'_>;
}

impl<R> ModifyDecor for R
where
    R: RawStringAccess,
{
    fn modify(&mut self) -> DecorFormatter<'_> {
        DecorFormatter::new(self)
    }
}

fn parse_multiline(input: &str) -> Option<Vec<DecorFragment>> {
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
    .ok()
}

fn parse_inline(input: &str) -> Option<Vec<DecorFragment>> {
    many0::<_, _, _, (), _>(alt((
        space1.map(DecorFragment::Whitespace),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorFragment::InlineComment),
    )))
    .parse(input)
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
