use hemtt_error::{make_source, thiserror, PrettyError, Source};
use hemtt_tokens::{Symbol, Token};

use crate::{
    defines::{Defines, DefinitionLibrary},
    parse::Rule,
};

#[derive(thiserror::Error, Debug)]
/// Errors that can occur during preprocessing
pub enum Error {
    #[error("Expected `{expected:?}`, found `{token:?}`,")]
    /// Expected a token, found something else
    UnexpectedToken {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The valid [`Symbol`]s that were expected
        expected: Vec<Symbol>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Unexpected EOF at `{token:?}`")]
    /// Unexpected end of file
    UnexpectedEOF {
        /// The token that was found
        token: Box<Token>,
    },
    #[error("Expected `{{ident}}`, found `{token:?}`, ")]
    /// Expected an identifier, found something else
    ExpectedIdent {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Unknown directive `{directive:?}`, ")]
    /// Unknown directive
    UnknownDirective {
        /// The [`Token`] that was found
        directive: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Function definition has multi-token arguments, `{token:?}`")]
    /// Tried to create a [`FunctionDefinition`](crate::context::FunctionDefinition) that has multi-token arguments
    ///
    /// ```cpp
    /// #define FUNC(my arg) ...
    /// ```
    DefineMultiTokenArgument {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Can not change built-in macros `{token:?}`")]
    /// Tried to change a built-in macro
    ChangeBuiltin {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Attempted to use `#if` on a unit or function macro, `{token:?}`")]
    /// Tried to use `#if` on a [`Unit`](crate::context::Definition::Unit) or [`FunctionDefinition`](crate::context::Definition::Function)
    IfUnitOrFunction {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Attempted to use `#if` on an undefined macro, `{token:?}`")]
    /// Tried to use `#if` on an undefined macro
    IfUndefined {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("Function call with incorrect number of arguments, expected `{expected}` got `{got}`. `{token:?}`")]
    /// Tried to call a [`FunctionDefinition`](crate::context::FunctionDefinition) with the wrong number of arguments
    FunctionCallArgumentCount {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The number of arguments that were expected
        expected: usize,
        /// The number of arguments that were found
        got: usize,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
        /// The defines at the point of the error
        defines: Defines,
    },
    #[error("Expected Function or Value, found Unit, `{token:?}`")]
    /// Tried to use a [`Unit`](crate::context::Definition::Unit) as a function or value
    ExpectedFunctionOrValue {
        /// The [`Token`] that was found
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("`#include` was encountered while using `NoResolver`")]
    /// Tried to use `#include` with [`NoResolver`](crate::resolver::resolvers::NoResolver)
    ResolveWithNoResolver {
        /// The [`Token`] stack trace
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("`#include` target `{target:?}` was not found")]
    /// The [`Resolver`](crate::resolver::Resolver) could not find the target
    IncludeNotFound {
        /// The target that was not found
        target: Vec<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(Box<std::io::Error>),
    #[error("Pest Error: {0}")]
    /// [`pest::error::Error`]
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
            Self::UnexpectedEOF { token } => {
                format!("Unexpected EOF near `{token:?}`,")
            }
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
                defines: _,
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
        match self {
            Self::FunctionCallArgumentCount {
                token,
                expected: _,
                got,
                trace: _,
                defines,
            } => {
                let Symbol::Word(function) = token.symbol() else {
                    return None;
                };
                let did_you_mean = defines.similar_function(function, Some(*got));
                Some(format!("Did you mean `{}`?", did_you_mean.join("`, `")))
            }
            _ => None,
        }
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

    fn trace(&self) -> Vec<Source> {
        let trace = match self {
            Self::UnexpectedToken { trace, .. }
            | Self::ExpectedIdent { trace, .. }
            | Self::UnknownDirective { trace, .. }
            | Self::DefineMultiTokenArgument { trace, .. }
            | Self::ChangeBuiltin { trace, .. }
            | Self::IfUnitOrFunction { trace, .. }
            | Self::IfUndefined { trace, .. }
            | Self::FunctionCallArgumentCount { trace, .. }
            | Self::ExpectedFunctionOrValue { trace, .. }
            | Self::ResolveWithNoResolver { trace, .. }
            | Self::IncludeNotFound { trace, .. } => trace.clone(),
            _ => vec![],
        };
        trace
            .into_iter()
            .map(|t| make_source(&t, String::new()).unwrap()) // TODO remove unwrap
            .collect()
    }
}
