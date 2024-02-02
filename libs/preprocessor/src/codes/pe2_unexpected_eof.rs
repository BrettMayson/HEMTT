use std::sync::Arc;

use hemtt_common::reporting::{Code, Token};

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
    pub fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
