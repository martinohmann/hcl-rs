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
enum DecorFragment<'i> {
    Whitespace(&'i str),
    InlineComment(&'i str),
    HashLineComment(&'i str),
    DoubleSlashLineComment(&'i str),
}

#[derive(Debug)]
struct DecorFragments<'i>(Vec<DecorFragment<'i>>);

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
        let mut fragments = if self.multiline {
            parse_multiline(input)?
        } else {
            parse_inline(input)?
        };

        self.normalize_start(&mut fragments);
        self.normalize_end(&mut fragments);
        self.normalize_leading_newline(&mut fragments);
        fragments.into_cow_str()
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

    fn normalize_start(&self, fragments: &mut Vec<DecorFragment>) {
        if let (Some(DecorFragment::Whitespace(first)), second) =
            (fragments.get(0), fragments.get(1))
        {
            let trimmed = trim(first);
            let has_second = second.is_some();

            if trimmed.is_empty() {
                fragments.remove(0);
            } else {
                *fragments.first_mut().unwrap() = DecorFragment::Whitespace(trimmed);
            }

            if self.leading_space && has_second {
                fragments.insert(0, DecorFragment::Whitespace(" "));
            }
        }
    }

    fn normalize_end(&self, fragments: &mut Vec<DecorFragment>) {
        let len = fragments.len();

        if len > 1 && !matches!(&fragments[len - 2], DecorFragment::InlineComment(_)) {
            return;
        }

        if let Some(DecorFragment::Whitespace(last)) = fragments.last() {
            let trimmed = trim(last);

            if trimmed.is_empty() {
                fragments.remove(len - 1);
            } else {
                *fragments.last_mut().unwrap() = DecorFragment::Whitespace(trimmed);
            }

            if self.trailing_space {
                fragments.push(DecorFragment::Whitespace(" "));
            }
        }
    }

    fn normalize_leading_newline(&self, fragments: &mut Vec<DecorFragment>) {
        if !self.leading_newline {
            return;
        }

        if fragments.is_empty() {
            fragments.push(DecorFragment::Whitespace("\n"));
        } else {
            if let Some(DecorFragment::Whitespace(first)) = fragments.first() {
                if !first.starts_with('\n') {
                    fragments.insert(0, DecorFragment::Whitespace("\n"));
                }
            } else {
                fragments.insert(0, DecorFragment::Whitespace("\n"));
            }
        }
    }
}

impl<'i> DecorFragment<'i> {
    fn as_str(&self) -> &'i str {
        match self {
            DecorFragment::Whitespace(s)
            | DecorFragment::InlineComment(s)
            | DecorFragment::HashLineComment(s)
            | DecorFragment::DoubleSlashLineComment(s) => s,
        }
    }
}

impl<'i> DecorFragments<'i> {
    fn into_cow_str(mut self) -> Option<Cow<'i, str>> {
        match self.len() {
            0 => None,
            1 => Some(Cow::Borrowed(self.remove(0).as_str())),
            _ => Some(Cow::Owned(self.iter().map(DecorFragment::as_str).collect())),
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
