use crate::util::try_unescape;
use crate::{Error, Identifier, Result};
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

/// A template expression embeds a program written in the template sub-language as an expression.
///
/// This type wraps the raw template string representation. Refer to the documentation of the
/// [`template`][`crate::template`] module if you need to parse and further evaluate the raw
/// template.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TemplateExpr {
    /// A quoted template expression is delimited by quote characters (`"`) and defines a template
    /// as a single-line expression with escape characters. The raw template string may contain
    /// escape sequences.
    QuotedString(String),
    /// A heredoc template expression is introduced by a `<<` sequence and defines a template via a
    /// multi-line sequence terminated by a user-chosen delimiter. The raw template string in the
    /// heredoc may contain escape sequences.
    Heredoc(Heredoc),
}

impl TemplateExpr {
    /// Returns the template as a `&str`.
    pub(crate) fn as_str(&self) -> &str {
        match self {
            TemplateExpr::QuotedString(s) => s,
            TemplateExpr::Heredoc(heredoc) => &heredoc.template,
        }
    }
}

impl From<&str> for TemplateExpr {
    fn from(s: &str) -> Self {
        TemplateExpr::QuotedString(s.to_owned())
    }
}

impl From<String> for TemplateExpr {
    fn from(string: String) -> Self {
        TemplateExpr::QuotedString(string)
    }
}

impl From<Heredoc> for TemplateExpr {
    fn from(heredoc: Heredoc) -> Self {
        TemplateExpr::Heredoc(heredoc)
    }
}

impl fmt::Display for TemplateExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TemplateExpr::QuotedString(_) => f.write_str(&try_unescape(self.as_str())),
            TemplateExpr::Heredoc(_) => f.write_str(self.as_str()),
        }
    }
}

/// A heredoc template expression is introduced by a `<<` sequence and defines a template via a
/// multi-line sequence terminated by a user-chosen delimiter.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Heredoc {
    /// The delimiter identifier that denotes the heredoc start and end.
    pub delimiter: Identifier,
    /// The raw template contained in the heredoc.
    pub template: String,
    /// The heredoc strip mode.
    pub strip: HeredocStripMode,
}

impl Heredoc {
    /// Creates a new `Heredoc` with the provided delimiter and template body.
    pub fn new<T>(delimiter: Identifier, template: T) -> Heredoc
    where
        T: Into<String>,
    {
        Heredoc {
            delimiter,
            template: template.into(),
            strip: HeredocStripMode::default(),
        }
    }

    /// Sets the heredoc strip mode to use on the template.
    pub fn with_strip_mode(mut self, strip: HeredocStripMode) -> Heredoc {
        self.strip = strip;
        self
    }
}

/// The strip behaviour for the template contained in the heredoc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeredocStripMode {
    /// Do not strip leading whitespace.
    #[default]
    None,
    /// Any literal string at the start of each line is analyzed to find the minimum number
    /// of leading spaces, and then that number of prefix spaces is removed from all line-leading
    /// literal strings. The final closing marker may also have an arbitrary number of spaces
    /// preceding it on its line.
    Indent,
}

impl HeredocStripMode {
    /// Returns the string representation of the heredoc strip mode. This is the part before the
    /// delimiter identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            HeredocStripMode::None => "<<",
            HeredocStripMode::Indent => "<<-",
        }
    }
}

impl FromStr for HeredocStripMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "<<" => Ok(HeredocStripMode::None),
            "<<-" => Ok(HeredocStripMode::Indent),
            _ => Err(Error::new(format!("invalid heredoc strip mode: `{s}`"))),
        }
    }
}
