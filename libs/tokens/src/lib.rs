#![deny(clippy::all, clippy::nursery)]
#![warn(clippy::pedantic)]

pub mod position;
pub mod symbol;
pub mod whitespace;

use position::Position;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    symbol: Symbol,
    source: Position,
    trace: Vec<Self>,
}

impl Token {
    #[must_use]
    pub const fn new(symbol: Symbol, source: Position, trace: Vec<Self>) -> Self {
        Self {
            symbol,
            source,
            trace,
        }
    }

    #[must_use]
    pub fn builtin(trace: Vec<Self>) -> Self {
        Self {
            symbol: Symbol::Void,
            source: Position::builtin(),
            trace,
        }
    }

    #[must_use]
    pub fn ending_newline() -> Self {
        Self {
            symbol: Symbol::Newline,
            source: Position::builtin(),
            trace: Vec::new(),
        }
    }

    #[must_use]
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    #[must_use]
    pub const fn source(&self) -> &Position {
        &self.source
    }

    #[must_use]
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
