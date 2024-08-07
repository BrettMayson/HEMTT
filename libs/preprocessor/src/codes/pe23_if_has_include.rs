use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// Use of `#if __has_include`
///
/// ```cpp
/// #pragma hemtt flag unknown
/// ```
pub struct IfHasInclude {
    /// The [`Token`] of the code
    token: Box<Token>,
}

impl Code for IfHasInclude {
    fn ident(&self) -> &'static str {
        "PE23"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "use of `__has_include`".to_string()
    }

    fn note(&self) -> Option<String> {
        Some("use of `#if __has_include` will prevent HEMTT from rapifying the file".to_string())
    }

    fn help(&self) -> Option<String> {
        Some(String::from("use `#pragma hemtt flag pe23_ignore_has_include`\nto have HEMTT act as if the include was not found.\nThis will still prevent HEMTT from rapifying the file\nbut will allow the false branch to be validated."))
    }
}

impl IfHasInclude {
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
