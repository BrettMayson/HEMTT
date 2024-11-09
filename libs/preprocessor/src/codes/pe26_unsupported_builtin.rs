use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// Built-in macro is not supported by HEMTT
pub struct BuiltInNotSupported {
    /// The [`Token`] of the built-in macro
    token: Box<Token>,
}

impl Code for BuiltInNotSupported {
    fn ident(&self) -> &'static str {
        "PE26"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("built-in macro `{}` is not supported by HEMTT", self.token.symbol())
    }

    fn note(&self) -> Option<String> {
        Some("certain built-in macros can not be rapified at build time\nHEMTT does not support them to prevent unexpected behaviour".to_string())
    }
}

impl BuiltInNotSupported {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
