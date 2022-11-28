use hemtt_error::{make_source, thiserror, PrettyError, Source};
use hemtt_tokens::{symbol::Symbol, Token};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`,")]
    UnexpectedToken {
        token: Box<Token>,
        expected: Vec<Symbol>,
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Expected `{{ident}}`, found `{token:?}`, ")]
    ExpectedIdent { token: Box<Token> },
    #[error("Expected `{{number}}`, found `{token:?}`, ")]
    ExpectedNumber { token: Box<Token> },

    #[error("IO Error: {0}")]
    Io(Box<std::io::Error>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl PrettyError for Error {
    fn brief(&self) -> String {
        match self {
            Self::UnexpectedToken { token, expected } => {
                format!(
                    "Expected `{expected:?}`, found `{symbol:?}`,",
                    symbol = token.symbol(),
                    expected = expected
                )
            }
            Self::UnexpectedEOF => "Unexpected EOF".to_string(),
            Self::ExpectedIdent { token } => {
                format!(
                    "Expected `{{ident}}`, found `{symbol:?}`,",
                    symbol = token.symbol()
                )
            }
            Self::ExpectedNumber { token } => {
                format!(
                    "Expected `{{number}}`, found `{symbol:?}`,",
                    symbol = token.symbol()
                )
            }
            Self::Io(e) => {
                format!("IO Error: {e}")
            }
        }
    }

    fn details(&self) -> Option<String> {
        None
    }

    fn help(&self) -> Option<String> {
        match self {
            Self::UnexpectedToken { token, expected } => {
                println!("checking expected");
                if expected == &[Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)] {
                    println!("checking symbol");
                    if let Symbol::Word(_) = token.symbol() {
                        println!("providing help");
                        return Some("Did you forget to place quotes around a string? Or perhaps you forgot to define / import a value.".to_string());
                    }
                } else if expected == &[Symbol::Semicolon] {
                    return Some("Did you forget to place a semicolon at the end of a line? Or perhaps you are missing quotes around a string?".to_string());
                }
            }
            Self::ExpectedIdent { token: _ } => {
                return Some("Is something in quotes that shouldn't be?".to_string());
            }
            _ => (),
        }
        None
    }

    fn source(&self) -> Option<Box<Source>> {
        match self {
            Self::UnexpectedToken { token, expected } => {
                println!("error for unexpected token: {token:?}");
                make_source(token, format!("expected one of: {expected:?}"))
                    .ok()
                    .map(Box::new)
            }
            Self::ExpectedIdent { token } => make_source(token, "expected: <ident>".to_string())
                .ok()
                .map(Box::new),
            _ => None,
        }
    }
}
