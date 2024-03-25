use std::sync::Arc;

use hemtt_workspace::reporting::{diagnostic::Yellow, Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// Unexpected token
pub struct UnexpectedToken {
    /// The [`Token`] that was found
    token: Box<Token>,
    /// A vec of [`Token`]s that would be valid here
    expected: Vec<String>,
}

impl Code for UnexpectedToken {
    fn ident(&self) -> &'static str {
        "PE1"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "unexpected token".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unexpected token `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let mut diag = Diagnostic::new(self.ident(), self.message()).with_label(
            Label::primary(
                self.token.position().path().clone(),
                self.token.position().span(),
            )
            .with_message(self.label_message()),
        );
        if !self.expected.is_empty() {
            diag = diag.with_help(format!(
                "expected one of: {}",
                Yellow.paint(self.expected.join(" "))
            ));
        }
        Some(diag)
    }
}

impl UnexpectedToken {
    pub fn new(token: Box<Token>, expected: Vec<String>) -> Self {
        Self { token, expected }
    }

    pub fn code(token: Token, expected: Vec<String>) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), expected)))
    }
}
