use std::{borrow::Cow, fmt::Display};

use crate::position::Position;

use super::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// A token from the tokenizer
pub struct Token {
    symbol: Symbol,
    source: Position,
}

impl Token {
    #[must_use]
    /// Create a new token
    pub const fn new(symbol: Symbol, source: Position) -> Self {
        Self { symbol, source }
    }

    #[must_use]
    /// Get the [`Symbol`] of the token
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    #[must_use]
    /// Get the [`Position`] of the token
    pub const fn position(&self) -> &Position {
        &self.source
    }

    #[must_use]
    /// For writing to a file for later parsing, returns a Cow to avoid allocation when possible
    pub fn to_source(&self) -> Cow<'_, str> {
        self.symbol.to_cow()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}
