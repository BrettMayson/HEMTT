use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Token};

use crate::Error;

#[allow(unused)]
/// An unknown `#pragma` directive
///
/// ```cpp
/// #pragma hemtt unknown
/// ```
pub struct PragmaUnknown {
    /// The [`Token`] of the unknown directive
    token: Box<Token>,
}

impl Code for PragmaUnknown {
    fn ident(&self) -> &'static str {
        "PE19"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("unknown `{}` pragma command", self.token.symbol(),)
    }

    fn label_message(&self) -> String {
        "unknown #pragma command".to_string()
    }

    fn help(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), &["suppress"]);
        if similar.is_empty() {
            None
        } else {
            Some(format!(
                "did you mean {}?",
                similar
                    .iter()
                    .map(|s| format!("`{s}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }

    fn suggestion(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), &["suppress"]);
        if similar.is_empty() {
            None
        } else {
            Some(similar[0].to_string())
        }
    }
}

impl PragmaUnknown {
    #[must_use]
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    #[must_use]
    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
