use std::sync::Arc;

use hemtt_common::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// Expected an identifier, found something else
pub struct ExpectedIdent {
    /// The [`Token`] that was found
    token: Box<Token>,
}

impl Code for ExpectedIdent {
    fn ident(&self) -> &'static str {
        "PE3"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "expected identifier".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "expected identifier, found `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }
}

impl ExpectedIdent {
    pub fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
