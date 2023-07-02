use ariadne::{Label, Report, ReportKind, Source};
use hemtt_error::thiserror;
use tracing::error;

use crate::{defines::Defines, parse::Rule, Symbol, Token};

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
    #[error("Unknown directive `{token:?}`, ")]
    /// Unknown directive
    UnknownDirective {
        /// The [`Token`] that was found
        token: Box<Token>,
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
        /// Skipped tokens of Unit
        skipped: Vec<Token>,
    },
    #[error("`#include` was encountered while using `NoResolver`")]
    /// Tried to use `#include` with [`NoResolver`](crate::resolver::resolvers::NoResolver)
    ResolveWithNoResolver {
        /// The [`Token`] stack trace
        token: Box<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("`#include` target `{token:?}` was not found")]
    /// The [`Resolver`](crate::resolver::Resolver) could not find the target
    IncludeNotFound {
        /// The target that was not found
        token: Vec<Token>,
        /// The [`Token`] stack trace
        trace: Vec<Token>,
    },
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(Box<std::io::Error>),
    #[error("Pest Error: {0}")]
    /// [`pest::error::Error`]
    Pest(Box<pest::error::Error<Rule>>),
    #[error("Vfs Error: {0}")]
    /// [`vfs::Error`]
    Vfs(Box<vfs::error::VfsError>),
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

impl From<vfs::error::VfsError> for Error {
    fn from(e: vfs::error::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}

impl Error {
    #[must_use]
    /// Create a user friendly report
    pub fn report(&self) -> String {
        let Some(token) = self.token() else {
            return self.to_string();
        };
        let span = if let Self::IncludeNotFound { token, .. } = self {
            token.first().unwrap().source().start().0..token.last().unwrap().source().end().0
        } else {
            token.source().start().0..token.source().end().0
        };
        let mut out = Vec::new();
        if let Err(e) = Report::build(
            ReportKind::Error,
            token.source().path_or_builtin(),
            span.start,
        )
        .with_message("no bueno")
        .with_label(Label::new((
            token.source().path_or_builtin(),
            span.start..span.end,
        )))
        .finish()
        .write_for_stdout(
            (
                token.source().path_or_builtin(),
                Source::from(token.source().path().map_or_else(String::new, |path| {
                    path.read_to_string().unwrap_or_default()
                })),
            ),
            &mut out,
        ) {
            error!("while reporting: {e}");
            return self.to_string();
        }
        String::from_utf8(out).unwrap_or_default()
    }

    #[must_use]
    /// Get the [`Token`] that caused the error
    pub fn token(&self) -> Option<&Token> {
        match self {
            Self::UnexpectedToken { token, .. } => Some(token.as_ref()),
            Self::UnexpectedEOF { token, .. } => Some(token.as_ref()),
            Self::ExpectedIdent { token, .. } => Some(token.as_ref()),
            Self::UnknownDirective { token, .. } => Some(token.as_ref()),
            Self::DefineMultiTokenArgument { token, .. } => Some(token.as_ref()),
            Self::ChangeBuiltin { token, .. } => Some(token.as_ref()),
            Self::IfUnitOrFunction { token, .. } => Some(token.as_ref()),
            Self::IfUndefined { token, .. } => Some(token.as_ref()),
            Self::FunctionCallArgumentCount { token, .. } => Some(token.as_ref()),
            Self::ExpectedFunctionOrValue { token, .. } => Some(token.as_ref()),
            Self::ResolveWithNoResolver { token, .. } => Some(token.as_ref()),
            Self::IncludeNotFound { token, .. } => token.first(),
            Self::Io(_) => None,
            Self::Pest(_) => None,
            Self::Vfs(_) => None,
        }
    }
}
