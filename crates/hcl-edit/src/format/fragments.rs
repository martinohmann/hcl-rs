use super::{Formatter, Indent};
use crate::{util::dedent, RawString};
use std::borrow::Cow;
use winnow::{
    branch::alt,
    bytes::{take_until0, take_while1},
    character::{not_line_ending, space1},
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
    Space,
    LineBreaks(&'a str),
    InlineComment(&'a str),
    LineComment(&'a str),
}

#[derive(Debug, Default, Clone)]
struct DecorFragments<'a>(Vec<DecorFragment<'a>>);

impl<'a> DecorFragment<'a> {
    fn indent(&self, prefix: &str, skip_first_line: bool) -> Cow<'a, str> {
        match self {
            DecorFragment::Space => reindent(" ", prefix, skip_first_line),
            DecorFragment::LineBreaks(s) => Cow::Owned(format!("{s}{prefix}")),
            DecorFragment::LineComment(s) => reindent(*s, prefix, skip_first_line),
            DecorFragment::InlineComment(s) if skip_first_line => Cow::Borrowed(s),
            DecorFragment::InlineComment(s) => {
                let mut lines = s.lines();

                let first_line = lines
                    .next()
                    .expect("inline comments always have at least one line");

                // Since inline comments can contain significant newline characters, we must only
                // indent the first line.
                let mut indented = reindent(first_line, prefix, false);

                let indented_mut = indented.to_mut();

                // Append the rest of the inline comment without altering the existing indent.
                for line in lines {
                    indented_mut.push('\n');
                    indented_mut.push_str(line);
                }

                indented
            }
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

    fn leading_newline(&mut self) {
        match self.0.first() {
            Some(DecorFragment::LineBreaks(_)) => {}
            _ => {
                self.0.insert(0, DecorFragment::LineBreaks("\n"));
            }
        }
    }

    fn pad(&mut self, padding: Padding) {
        match padding {
            Padding::None => {}
            Padding::Start => self.pad_start(),
            Padding::End => self.pad_end(),
            Padding::Both => self.pad_both(),
        }
    }

    fn pad_both(&mut self) {
        if self.0.is_empty() {
            self.0.push(DecorFragment::Space);
        } else {
            self.pad_start();
            self.pad_end();
        }
    }

    fn pad_start(&mut self) {
        if let Some(DecorFragment::InlineComment(_) | DecorFragment::LineComment(_)) =
            self.0.first()
        {
            self.0.insert(0, DecorFragment::Space);
        }
    }

    fn pad_end(&mut self) {
        if let Some(DecorFragment::InlineComment(_)) = self.0.last() {
            self.0.push(DecorFragment::Space);
        }
    }

    fn remove_insignificant_spaces(&mut self) {
        let mut remove_space = true;

        // Remove leading space and spaces immediately preceded by line breaks.
        self.0.retain(|fragment| {
            if let DecorFragment::Space = fragment {
                let keep_space = std::mem::replace(&mut remove_space, false);
                !keep_space
            } else {
                remove_space = matches!(fragment, DecorFragment::LineBreaks(_));
                true
            }
        });

        // Remove potential trailing space after an inline comment.
        if let Some(DecorFragment::Space) = self.0.last() {
            self.0.pop();
        }
    }

    fn indent(&self, prefix: &str, mut skip_first_line: bool) -> RawString {
        if self.0.is_empty() && !skip_first_line {
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

pub(super) struct DecorFormatter<'a> {
    raw: &'a mut dyn RawStringAccess,
    kind: DecorKind,
    leading_newline: bool,
    indent_first_line: Option<bool>,
    padding: Padding,
}

impl<'a> DecorFormatter<'a> {
    fn new(raw: &'a mut dyn RawStringAccess) -> DecorFormatter<'a> {
        DecorFormatter {
            raw,
            kind: DecorKind::Multiline,
            leading_newline: false,
            indent_first_line: None,
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

    pub(super) fn indent_first_line(mut self, yes: bool) -> Self {
        self.indent_first_line = Some(yes);
        self
    }

    pub(super) fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub(super) fn format(self, formatter: &mut Formatter) {
        let mut fragments = DecorFragments::parse(self.raw.get(), self.kind).unwrap_or_default();

        fragments.remove_insignificant_spaces();

        if self.leading_newline {
            fragments.leading_newline();
        }

        fragments.pad(self.padding);

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
        take_while1(is_line_break).map(DecorFragment::LineBreaks),
        space1.value(DecorFragment::Space),
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
        space1.value(DecorFragment::Space),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorFragment::InlineComment),
    )))
    .parse(input)
    .ok()
}

fn is_line_break(ch: char) -> bool {
    ch == '\n' || ch == '\r'
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

    if s.is_empty() && !skip_first_line {
        return Cow::Owned(prefix.to_owned());
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
        output.push_str(prefix);
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
                "    // foo \n    /* bar\nbaz */ # 123\n    #456",
                DecorKind::Multiline,
            ),
            (
                "  /* bar\nbaz */ \t \t /* qux */",
                "    /* bar\nbaz */ /* qux */",
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
