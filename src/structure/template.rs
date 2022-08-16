use super::Identifier;
use crate::{parser::dedent_string, template::Template, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A quoted template expression is delimited by quote characters (`"`) and defines a template as
/// a single-line expression with escape characters.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "$hcl::template_expr")]
pub enum TemplateExpr {
    /// A quoted string template.
    QuotedString(String),
    /// A heredoc template.
    Heredoc(Heredoc),
}

impl TemplateExpr {
    /// Parses the template expression and returns the template. This will return an error if the
    /// template expression contains invalid template syntax.
    pub fn to_template(&self) -> Result<Template> {
        match self {
            TemplateExpr::QuotedString(s) => s.parse(),
            TemplateExpr::Heredoc(heredoc) => heredoc.to_string().parse(),
        }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateExpr::QuotedString(string) => fmt::Display::fmt(string, f),
            TemplateExpr::Heredoc(heredoc) => fmt::Display::fmt(heredoc, f),
        }
    }
}

/// A heredoc template expression is introduced by a `<<` sequence and defines a template via a
/// multi-line sequence terminated by a user-chosen delimiter.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "$hcl::heredoc")]
pub struct Heredoc {
    /// The delimiter identifier that denotes the heredoc start and end.
    pub delimiter: Identifier,
    /// The raw template contained in the heredoc.
    pub template: String,
    /// The heredoc strip mode.
    pub strip: HeredocStripMode,
}

impl fmt::Display for Heredoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.strip {
            HeredocStripMode::None => f.write_str(&self.template),
            HeredocStripMode::Indent => {
                let dedented = dedent_string(&self.template);
                f.write_str(&dedented)
            }
        }
    }
}

/// The strip behaviour for the template contained in the heredoc.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename = "$hcl::heredoc_strip")]
pub enum HeredocStripMode {
    /// `<<`: Do not strip leading whitespace.
    None,
    /// `<<-`: Any literal string at the start of each line is analyzed to find the minimum number
    /// of leading spaces, and then that number of prefix spaces is removed from all line-leading
    /// literal strings. The final closing marker may also have an arbitrary number of spaces
    /// preceding it on its line.
    Indent,
}
