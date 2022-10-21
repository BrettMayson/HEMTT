pub mod position;
pub mod symbol;
pub mod whitespace;

use position::Position;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    symbol: Symbol,
    source: Position,
}

impl Token {
    pub const fn new(symbol: Symbol, source: Position) -> Self {
        Self { symbol, source }
    }

    pub fn builtin() -> Self {
        Self {
            symbol: Symbol::Void,
            source: Position::builtin(),
        }
    }

    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub const fn source(&self) -> &Position {
        &self.source
    }

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
