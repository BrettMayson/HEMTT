use std::path::Path;

use hemtt_error::{make_source, read_lines_from_file, PrettyError, Source};
use hemtt_tokens::{symbol::Symbol, Token};

use crate::parse::Rule;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`,")]
    UnexpectedToken { token: Token, expected: Vec<Symbol> },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Expected `{{ident}}`, found `{token:?}`, ")]
    ExpectedIdent { token: Token },
    #[error("Unknown directive `{directive:?}`, ")]
    UnknownDirective { directive: Token },
    #[error("Function definition has multi-token arguments, `{token:?}`")]
    DefineMultiTokenArgument { token: Token },
    #[error("Can not change built-in macros `{token:?}`")]
    ChangeBuiltin { token: Token },
    #[error("Attempted to use `#if` on a unit or function macro, `{token:?}`")]
    IfUnitOrFunction { token: Token },
    #[error("Attempted to use `#if` on an undefined macro, `{token:?}`")]
    IfUndefined { token: Token },
    #[error("Function call with incorrect number of arguments, expected `{expected}` got `{got}`. `{token:?}`")]
    FunctionCallArgumentCount {
        token: Token,
        expected: usize,
        got: usize,
    },
    #[error("Expected Function or Value, found Unit, `{token:?}`")]
    ExpectedFunctionOrValue { token: Token },
    #[error("`#include` was encountered while using `NoResolver`")]
    ResolveWithNoResolver,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Pest Error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
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
            Self::UnknownDirective { directive } => {
                format!(
                    "Unknown directive `{directive:?}`,",
                    directive = directive.symbol()
                )
            }
            Self::DefineMultiTokenArgument { .. } => {
                "Function definition has multi-token arguments".to_string()
            }
            Self::ChangeBuiltin { token } => {
                format!(
                    "Can not change built-in macros `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::IfUnitOrFunction { token } => {
                format!(
                    "Attempted to use `#if` on a unit or function macro, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::IfUndefined { token } => {
                format!(
                    "Attempted to use `#if` on an undefined macro, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::FunctionCallArgumentCount {
                token,
                expected,
                got,
            } => {
                format!("Function call with incorrect number of arguments, expected `{expected}` got `{got}`. `{symbol:?}`", symbol = token.symbol(), expected = expected, got = got)
            }
            Self::ExpectedFunctionOrValue { token } => {
                format!(
                    "Expected Function or Value, found Unit, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::ResolveWithNoResolver => {
                "`#include` was encountered while using `NoResolver`".to_string()
            }
            Self::Io(e) => {
                format!("IO Error: {}", e)
            }
            Self::Pest(e) => {
                format!("Pest Error: {}", e)
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
            Self::UnexpectedEOF => None,
            Self::ExpectedIdent { token } => {
                Some(make_source(token, "expected an identifier".to_string()))
            }
            Self::UnknownDirective { directive } => {
                Some(make_source(directive, "unknown directive".to_string()))
            }
            Self::DefineMultiTokenArgument { token } => {
                Some(make_source(token, "invalid arguments".to_string()))
            }
            Self::ChangeBuiltin { token } => Some(make_source(token, "build-in macro".to_string())),
            Self::IfUnitOrFunction { token } => {
                Some(make_source(token, "invalid macro type".to_string()))
            }
            Self::IfUndefined { token } => {
                Some(make_source(token, "macro is undefined".to_string()))
            }
            Self::FunctionCallArgumentCount {
                token, expected, ..
            } => Some(make_source(
                token,
                format!("Expects {} arguments", expected),
            )),
            Self::ExpectedFunctionOrValue { token } => {
                Some(make_source(token, "expects function or value".to_string()))
            }
            Self::ResolveWithNoResolver => None,
            Self::Io(_) => None,
            Self::Pest(_) => None,
        }
    }
}
