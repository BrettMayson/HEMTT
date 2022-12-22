#![deny(clippy::all, clippy::nursery, missing_docs)]
#![warn(clippy::pedantic)]

//! HEMTT - Arma 3 Tokenizer

mod position;
mod symbol;
pub mod whitespace;

pub use position::Position;
pub use symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A token from the tokenizer
pub struct Token {
    symbol: Symbol,
    source: Position,
    parent: Option<Box<Self>>,
}

impl Token {
    #[must_use]
    /// Create a new token
    pub const fn new(symbol: Symbol, source: Position, parent: Option<Box<Self>>) -> Self {
        Self {
            symbol,
            source,
            parent,
        }
    }

    #[must_use]
    /// Create a new token built-in token
    pub fn builtin(parent: Option<Box<Self>>) -> Self {
        Self {
            symbol: Symbol::Void,
            source: Position::builtin(),
            parent,
        }
    }

    #[must_use]
    /// Create a newline token
    pub fn ending_newline(parent: Option<Box<Self>>) -> Self {
        Self {
            symbol: Symbol::Newline,
            source: Position::builtin(),
            parent,
        }
    }

    #[must_use]
    /// Get the [`Symbol`] of the token
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    #[must_use]
    /// Get the [`Position`] of the token
    pub const fn source(&self) -> &Position {
        &self.source
    }

    #[must_use]
    /// Get the parent token
    pub const fn parent(&self) -> &Option<Box<Self>> {
        &self.parent
    }

    /// Set the parent token
    pub fn set_parent(&mut self, parent: Option<Box<Self>>) {
        self.parent = parent;
    }

    #[must_use]
    /// Get the string value of a [`Symbol::Word`] token
    pub const fn word(&self) -> Option<&String> {
        if let Symbol::Word(word) = &self.symbol {
            Some(word)
        } else {
            None
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        self.symbol.to_string()
    }
}
