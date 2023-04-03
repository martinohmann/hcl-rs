use std::borrow::Cow;
use std::ops;
use winnow::{
    branch::alt,
    bytes::take_until0,
    character::{multispace1, not_line_ending, space1},
    multi::many0,
    Parser,
};

use crate::{repr::Decor, RawString};

#[derive(Debug)]
enum DecorPart<'i> {
    Whitespace(&'i str),
    InlineComment(&'i str),
    HashLineComment(&'i str),
    DoubleSlashLineComment(&'i str),
}

#[derive(Debug)]
struct DecorParts<'i>(Vec<DecorPart<'i>>);

#[derive(Default, Debug, Clone)]
pub(crate) struct Normalizer {
    leading_space: bool,
    trailing_space: bool,
    multiline: bool,
    leading_newline: bool,
}

impl Normalizer {
    pub fn new() -> Self {
        Normalizer::default()
    }

    pub fn leading_space(mut self) -> Self {
        self.leading_space = true;
        self
    }

    pub fn trailing_space(mut self) -> Self {
        self.trailing_space = true;
        self
    }

    pub fn leading_newline(mut self) -> Self {
        self.leading_newline = true;
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn normalize<'i>(&self, input: &'i str) -> Option<Cow<'i, str>> {
        let mut parts = if self.multiline {
            parse_multiline(input)?
        } else {
            parse_inline(input)?
        };

        self.normalize_start(&mut parts);
        self.normalize_end(&mut parts);
        self.normalize_leading_newline(&mut parts);
        parts.into_cow_str()
    }

    pub fn normalize_raw(&self, input: &RawString) -> Option<RawString> {
        self.normalize(&input).map(Into::into)
    }

    pub fn normalize_decor_prefix(&self, decor: &mut Decor) {
        if let Some(normalized) = decor
            .take_prefix()
            .and_then(|prefix| self.normalize_raw(&prefix))
        {
            decor.set_prefix(normalized);
        }
    }

    pub fn normalize_decor_suffix(&self, decor: &mut Decor) {
        if let Some(normalized) = decor
            .take_suffix()
            .and_then(|suffix| self.normalize_raw(&suffix))
        {
            decor.set_suffix(normalized);
        }
    }

    fn normalize_start(&self, parts: &mut Vec<DecorPart>) {
        if let (Some(DecorPart::Whitespace(first)), second) = (parts.get(0), parts.get(1)) {
            let trimmed = trim(first);
            let has_second = second.is_some();

            if trimmed.is_empty() {
                parts.remove(0);
            } else {
                *parts.first_mut().unwrap() = DecorPart::Whitespace(trimmed);
            }

            if self.leading_space && has_second {
                parts.insert(0, DecorPart::Whitespace(" "));
            }
        }
    }

    fn normalize_end(&self, parts: &mut Vec<DecorPart>) {
        let len = parts.len();

        if len > 1 && !matches!(&parts[len - 2], DecorPart::InlineComment(_)) {
            return;
        }

        if let Some(DecorPart::Whitespace(last)) = parts.last() {
            let trimmed = trim(last);

            if trimmed.is_empty() {
                parts.remove(len - 1);
            } else {
                *parts.last_mut().unwrap() = DecorPart::Whitespace(trimmed);
            }

            if self.trailing_space {
                parts.push(DecorPart::Whitespace(" "));
            }
        }
    }

    fn normalize_leading_newline(&self, parts: &mut Vec<DecorPart>) {
        if !self.leading_newline {
            return;
        }

        if parts.is_empty() {
            parts.push(DecorPart::Whitespace("\n"));
        } else {
            if let Some(DecorPart::Whitespace(first)) = parts.first() {
                if !first.starts_with('\n') {
                    parts.insert(0, DecorPart::Whitespace("\n"));
                }
            } else {
                parts.insert(0, DecorPart::Whitespace("\n"));
            }
        }
    }
}

impl<'i> DecorPart<'i> {
    fn as_str(&self) -> &'i str {
        match self {
            DecorPart::Whitespace(s)
            | DecorPart::InlineComment(s)
            | DecorPart::HashLineComment(s)
            | DecorPart::DoubleSlashLineComment(s) => s,
        }
    }
}

impl<'i> DecorParts<'i> {
    fn into_cow_str(mut self) -> Option<Cow<'i, str>> {
        match self.len() {
            0 => None,
            1 => Some(Cow::Borrowed(self.remove(0).as_str())),
            _ => Some(Cow::Owned(self.iter().map(DecorPart::as_str).collect())),
        }
    }
}

impl<'i> ops::Deref for DecorParts<'i> {
    type Target = Vec<DecorPart<'i>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'i> ops::DerefMut for DecorParts<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn parse_multiline(input: &str) -> Option<DecorParts> {
    many0::<_, _, _, (), _>(alt((
        multispace1.map(DecorPart::Whitespace),
        ('#', not_line_ending)
            .recognize()
            .map(DecorPart::HashLineComment),
        ("//", not_line_ending)
            .recognize()
            .map(DecorPart::DoubleSlashLineComment),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorPart::InlineComment),
    )))
    .parse(input)
    .map(DecorParts)
    .ok()
}

fn parse_inline(input: &str) -> Option<DecorParts> {
    many0::<_, _, _, (), _>(alt((
        space1.map(DecorPart::Whitespace),
        ("/*", take_until0("*/"), "*/")
            .recognize()
            .map(DecorPart::InlineComment),
    )))
    .parse(input)
    .map(DecorParts)
    .ok()
}

fn is_space(ch: char) -> bool {
    ch.is_whitespace() && ch != '\r' && ch != '\n'
}

fn trim(s: &str) -> &str {
    let s = s.trim_start_matches(is_space);
    let s = s.trim_end_matches(is_space);

    s.strip_prefix("\r\n")
        .or_else(|| s.strip_prefix('\n'))
        .unwrap_or(s)
}

fn trim_start(s: &str) -> &str {
    let s = s.trim_start_matches(is_space);

    s.strip_prefix("\r\n")
        .or_else(|| s.strip_prefix('\n'))
        .unwrap_or(s)
}

fn trim_end(s: &str) -> &str {
    let s = s.trim_end_matches(is_space);

    s.strip_suffix("\r\n")
        .or_else(|| s.strip_suffix('\n'))
        .unwrap_or(s)
}
