use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// __EXEC is not a supported macro
pub struct ExecNotSupported {
    /// The [`Token`] of the __EXEC macro
    token: Box<Token>,
}

impl Code for ExecNotSupported {
    fn ident(&self) -> &'static str {
        "PE25"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "__EXEC is not supported".to_string()
    }
}

impl ExecNotSupported {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
