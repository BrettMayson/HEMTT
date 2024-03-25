use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// Unknown directive
pub struct UnknownDirective {
    /// The [`Token`] that was found
    token: Box<Token>,
}

impl Code for UnknownDirective {
    fn ident(&self) -> &'static str {
        "PE4"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "unknown directive".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "unknown directive `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }
}

impl UnknownDirective {
    pub fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
