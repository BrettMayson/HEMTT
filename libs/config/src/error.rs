use std::path::Path;

use hemtt_error::{make_source, read_lines_from_file, PrettyError, Source};
use hemtt_tokens::{symbol::Symbol, Token};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`,")]
    UnexpectedToken { token: Token, expected: Vec<Symbol> },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Expected `{{ident}}`, found `{token:?}`, ")]
    ExpectedIdent { token: Token },

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
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
            Self::Io(e) => {
                format!("IO Error: {}", e)
            }
        }
    }

    fn details(&self) -> Option<String> {
        None
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn source(&self) -> Option<Source> {
        match self {
            Self::UnexpectedToken { token, expected } => Some(make_source(
                token,
                format!("expected one of: {:?}", expected),
            )),
            Self::ExpectedIdent { token } => {
                Some(make_source(token, "expected: <ident>".to_string()))
            }
            _ => None,
        }
    }
}
