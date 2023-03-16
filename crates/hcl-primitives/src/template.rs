//! Primitives for the HCL template sub-language.

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
