use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Tried to use a [`Unit`](crate::context::Definition::Unit) as a function or value
pub struct ExpectedFunctionOrValue {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// The [`Token`] of the function
    source: Box<Token>,
    /// Likely a function
    likely_function: bool,
}

impl Code for ExpectedFunctionOrValue {
    fn ident(&self) -> &'static str {
        "PE11"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        if self.likely_function {
            "attempted to use a unit as a function".to_string()
        } else {
            "attempted to use a unit as a value".to_string()
        }
    }

    fn label_message(&self) -> String {
        if self.likely_function {
            "used as a function".to_string()
        } else {
            "used as a value".to_string()
        }
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_labels(vec![Label::secondary(
            self.source.position().path().clone(),
            self.source.position().span(),
        )
        .with_message("defined as a unit")])
    }
}

impl ExpectedFunctionOrValue {
    pub fn new(token: Box<Token>, source: Box<Token>, likely_function: bool) -> Self {
        Self {
            token,
            source,
            likely_function,
        }
    }

    pub fn code(token: Token, source: Token, likely_function: bool) -> Error {
        Error::Code(Arc::new(Self::new(
            Box::new(token),
            Box::new(source),
            likely_function,
        )))
    }
}
