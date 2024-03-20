use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Tried to create a [`FunctionDefinition`](crate::context::FunctionDefinition) that has multi-token arguments
///
/// ```cpp
/// #define FUNC(my arg) ...
/// ```
pub struct DefineMissingComma {
    /// The [`Token`] of the current arg
    current: Box<Token>,
    /// The [`Token`] of the previous arg
    previous: Box<Token>,
}

impl Code for DefineMissingComma {
    fn ident(&self) -> &'static str {
        "PE5"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.current)
    }

    fn message(&self) -> String {
        "define arguments missing comma".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "define arguments missing comma `{}`",
            self.current.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn help(&self) -> Option<String> {
        Some("define arguments must be separated by a comma".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{}, {}",
            self.previous.symbol(),
            self.current.symbol()
        ))
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_label(
            Label::secondary(
                self.current.position().path().clone(),
                self.previous.position().span().end..self.current.position().span().start,
            )
            .with_message("missing comma"),
        )
    }
}

impl DefineMissingComma {
    pub fn new(current: Box<Token>, previous: Box<Token>) -> Self {
        Self { current, previous }
    }

    pub fn code(current: Token, previous: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(current), Box::new(previous))))
    }
}
