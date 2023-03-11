//! Primitives for the HCL template sub-language.

/// Controls the whitespace strip behaviour on adjacent string literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strip {
    /// Don't strip adjacent spaces.
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
    pub fn strip_start(self) -> bool {
        matches!(self, Strip::Start | Strip::Both)
    }

    /// Returns `true` if adjacent spaces should be stripped from an immediately following string
    /// literal.
    pub fn strip_end(self) -> bool {
        matches!(self, Strip::End | Strip::Both)
    }
}

impl Default for Strip {
    fn default() -> Strip {
        Strip::None
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
