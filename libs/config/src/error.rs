use hemtt_error::{make_source, thiserror, PrettyError, Source};
use hemtt_tokens::{Symbol, Token};

#[derive(thiserror::Error, Debug)]
/// Error type for the config parser
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`")]
    /// Expected a different token in the current context
    UnexpectedToken {
        /// The token that was found
        token: Box<Token>,
        /// The token that was expected
        expected: Vec<Symbol>,
    },
    #[error("Unexpected EOF at `{token:?}`")]
    /// Unexpected end of file
    UnexpectedEOF {
        /// The token that was found
        token: Box<Token>,
    },
    #[error("Expected `{{ident}}`, found `{token:?}`")]
    /// Expected an identifier in the current context
    ExpectedIdent {
        /// The token that was found
        token: Box<Token>,
    },
    #[error("Expected `{{number}}`, found `{token:?}`")]
    /// Expected a number in the current context
    ExpectedNumber {
        /// The token that was found
        token: Box<Token>,
    },

    #[error("IO Error: {0}")]
    /// [std::io::Error]
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
            Self::UnexpectedEOF { token } => {
                format!("Unexpected EOF near `{token:?}`,")
            }
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
                if expected == &[Symbol::LeftBrace, Symbol::DoubleQuote, Symbol::Digit(0)] {
                    if let Symbol::Word(_) = token.symbol() {
                        return Some("Did you forget to place quotes around a string? Or perhaps you forgot to define / import a value.".to_string());
                    }
                    if token.symbol() == &Symbol::Escape {
                        return Some("Did you forget to place quotes around a string? Or perhaps you forgot to use Q infront of a path macro.".to_string());
                    }
                } else if expected == &[Symbol::Semicolon] {
                    return Some("Did you forget to place a semicolon at the end of a line? Or perhaps you are missing quotes around a string?".to_string());
                }
            }
            Self::ExpectedIdent { token: _ } => {
                return Some("Is something quoted that shouldn't be?".to_string());
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
            Self::ExpectedNumber { token } => make_source(token, "expected: <number>".to_string())
                .ok()
                .map(Box::new),
            _ => None,
        }
    }

    fn trace(&self) -> Vec<Source> {
        let mut parent = match self {
            Self::ExpectedIdent { token }
            | Self::UnexpectedToken { token, expected: _ }
            | Self::ExpectedNumber { token }
            | Self::UnexpectedEOF { token } => token.parent(),
            Self::Io(_) => &None,
        };
        let mut trace = Vec::new();
        while let Some(p) = parent {
            parent = p.parent();
            let source = make_source(p, String::new());
            if let Ok(source) = source {
                trace.push(source);
            }
        }
        trace.reverse();
        trace
    }
}
