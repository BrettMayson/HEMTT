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
        None
    }

    fn source(&self) -> Option<Box<Source>> {
        match self {
            Self::UnexpectedToken { token, expected } => {
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
