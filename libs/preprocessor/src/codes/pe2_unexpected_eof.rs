use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

/// Unexpected end of file
pub struct UnexpectedEOF {
    /// The token that was found
    token: Box<Token>,
}

impl Code for UnexpectedEOF {
    fn ident(&self) -> &'static str {
        "PE2"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "unexpected end of file".to_string()
    }
}

impl UnexpectedEOF {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
