use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// #else read while not in a #if directive
///
/// ```cpp
/// #else
/// ```
pub struct UnexpectedElse {
    /// The [`Token`] of the `#else`
    token: Box<Token>,
}

impl Code for UnexpectedElse {
    fn ident(&self) -> &'static str {
        "PE28"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "#else when not in a #if directive".to_string()
    }
}

impl UnexpectedElse {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
