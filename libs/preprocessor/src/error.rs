use hemtt_error::{make_source, thiserror, PrettyError, Source};
use hemtt_tokens::{symbol::Symbol, Token};

use crate::parse::Rule;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`,")]
    UnexpectedToken {
        token: Box<Token>,
        expected: Vec<Symbol>,
        trace: Vec<Token>,
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Expected `{{ident}}`, found `{token:?}`, ")]
    ExpectedIdent {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Unknown directive `{directive:?}`, ")]
    UnknownDirective {
        directive: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Function definition has multi-token arguments, `{token:?}`")]
    DefineMultiTokenArgument {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Can not change built-in macros `{token:?}`")]
    ChangeBuiltin {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Attempted to use `#if` on a unit or function macro, `{token:?}`")]
    IfUnitOrFunction {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Attempted to use `#if` on an undefined macro, `{token:?}`")]
    IfUndefined {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("Function call with incorrect number of arguments, expected `{expected}` got `{got}`. `{token:?}`")]
    FunctionCallArgumentCount {
        token: Box<Token>,
        expected: usize,
        got: usize,
        trace: Vec<Token>,
    },
    #[error("Expected Function or Value, found Unit, `{token:?}`")]
    ExpectedFunctionOrValue {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("`#include` was encountered while using `NoResolver`")]
    ResolveWithNoResolver {
        token: Box<Token>,
        trace: Vec<Token>,
    },
    #[error("`#include` target `{target:?}` was not found")]
    IncludeNotFound {
        target: Vec<Token>,
        trace: Vec<Token>,
    },
    #[error("IO Error: {0}")]
    Io(Box<std::io::Error>),
    #[error("Pest Error: {0}")]
    Pest(Box<pest::error::Error<Rule>>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Self::Pest(Box::new(e))
    }
}

impl PrettyError for Error {
    fn brief(&self) -> String {
        match self {
            Self::UnexpectedToken {
                token,
                expected,
                trace: _,
            } => {
                format!(
                    "Expected `{expected:?}`, found `{symbol:?}`,",
                    symbol = token.symbol(),
                    expected = expected
                )
            }
            Self::UnexpectedEOF => "Unexpected EOF".to_string(),
            Self::ExpectedIdent { token, trace: _ } => {
                format!(
                    "Expected `{{ident}}`, found `{symbol:?}`,",
                    symbol = token.symbol()
                )
            }
            Self::UnknownDirective {
                directive,
                trace: _,
            } => {
                format!(
                    "Unknown directive `{directive:?}`,",
                    directive = directive.symbol()
                )
            }
            Self::DefineMultiTokenArgument { .. } => {
                "Function definition has multi-token arguments".to_string()
            }
            Self::ChangeBuiltin { token, trace: _ } => {
                format!(
                    "Can not change built-in macros `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::IfUnitOrFunction { token, trace: _ } => {
                format!(
                    "Attempted to use `#if` on a unit or function macro, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::IfUndefined { token, trace: _ } => {
                format!(
                    "Attempted to use `#if` on an undefined macro, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::FunctionCallArgumentCount {
                token,
                expected,
                got,
                trace: _,
            } => {
                format!("Function call with incorrect number of arguments, expected `{expected}` got `{got}`. `{symbol:?}`", symbol = token.symbol())
            }
            Self::ExpectedFunctionOrValue { token, trace: _ } => {
                format!(
                    "Expected Function or Value, found Unit, `{symbol:?}`",
                    symbol = token.symbol()
                )
            }
            Self::ResolveWithNoResolver { token: _, trace: _ } => {
                "`#include` was encountered while using `NoResolver`".to_string()
            }
            Self::IncludeNotFound { target, trace: _ } => {
                let target = target
                    .iter()
                    .map(|t| t.symbol().to_string())
                    .collect::<String>();
                format!("`#include` target `{target:?}` was not found")
            }
            Self::Io(e) => {
                format!("IO Error: {e}")
            }
            Self::Pest(e) => {
                format!("Pest Error: {e}")
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
            Self::UnexpectedToken {
                token,
                expected,
                trace: _,
            } => make_source(token, format!("expected one of: {expected:?}"))
                .ok()
                .map(Box::new),
            Self::ExpectedIdent { token, trace: _ } => {
                make_source(token, "expected an identifier".to_string())
                    .ok()
                    .map(Box::new)
            }
            Self::UnknownDirective {
                directive,
                trace: _,
            } => make_source(directive, "unknown directive".to_string())
                .ok()
                .map(Box::new),
            Self::DefineMultiTokenArgument { token, trace: _ } => {
                make_source(token, "invalid arguments".to_string())
                    .ok()
                    .map(Box::new)
            }
            Self::ChangeBuiltin { token, trace: _ } => {
                make_source(token, "build-in macro".to_string())
                    .ok()
                    .map(Box::new)
            }
            Self::IfUnitOrFunction { token, trace: _ } => {
                make_source(token, "invalid macro type".to_string())
                    .ok()
                    .map(Box::new)
            }
            Self::IfUndefined { token, trace: _ } => {
                make_source(token, "macro is undefined".to_string())
                    .ok()
                    .map(Box::new)
            }
            Self::FunctionCallArgumentCount {
                token, expected, ..
            } => make_source(token, format!("Expects {expected} arguments"))
                .ok()
                .map(Box::new),
            Self::ExpectedFunctionOrValue { token, trace: _ } => {
                make_source(token, "expects function or value".to_string())
                    .ok()
                    .map(Box::new)
            }
            _ => None,
        }
    }
}
