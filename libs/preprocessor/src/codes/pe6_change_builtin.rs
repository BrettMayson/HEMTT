use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic, Token};

use crate::Error;

#[allow(unused)]
/// Tried to change a built-in macro
pub struct ChangeBuiltin {
    /// The [`Token`] that was found
    token: Box<Token>,
}

impl Code for ChangeBuiltin {
    fn ident(&self) -> &'static str {
        "PE6"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "tried changing a built-in macro".to_string()
    }

    fn label_message(&self) -> String {
        format!(
            "built-in macro `{}`",
            self.token.symbol().to_string().replace('\n', "\\n")
        )
    }

    fn note(&self) -> Option<String> {
        Some("built-in macros cannot be changed".to_string())
    }
}

impl ChangeBuiltin {
    pub fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
