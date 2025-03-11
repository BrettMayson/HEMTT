use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// #endif read while not in a #if directive
///
/// ```cpp
/// #endif
/// ```
pub struct UnexpectedEndif {
    /// The [`Token`] of the `#endif`
    token: Box<Token>,
}

impl Code for UnexpectedEndif {
    fn ident(&self) -> &'static str {
        "PE27"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "#endif when not in a #if directive".to_string()
    }
}

impl UnexpectedEndif {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
