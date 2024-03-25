use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Tried to use a [`FunctionDefinition`](crate::context::FunctionDefinition) as a value
pub struct FunctionAsValue {
    /// The [`Token`] that was called
    token: Box<Token>,
    /// The [`Token`] of the function
    source: Box<Token>,
    /// The report
    report: Option<String>,
}

impl Code for FunctionAsValue {
    fn ident(&self) -> &'static str {
        "PE10"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "attempted to use a function as a value".to_string()
    }

    fn label_message(&self) -> String {
        "used as a value".to_string()
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_labels(vec![Label::secondary(
            self.source.position().path().clone(),
            self.source.position().span(),
        )
        .with_message("defined as a function")])
    }
}

impl FunctionAsValue {
    pub fn new(token: Box<Token>, source: Box<Token>) -> Self {
        Self {
            token,
            source,
            report: None,
        }
    }

    pub fn code(token: Token, source: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), Box::new(source))))
    }
}
