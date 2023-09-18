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
    /// For writing to a file for later parsing
    pub fn to_source(&self) -> String {
        if self.symbol == Symbol::Join {
            String::new()
        } else {
            self.symbol.to_string()
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        self.symbol.to_string()
    }
}
