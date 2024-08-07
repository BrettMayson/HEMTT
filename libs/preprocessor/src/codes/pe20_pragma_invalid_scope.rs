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
pub struct PragmaInvalidScope {
    /// The [`Token`] of the scope
    token: Box<Token>,
    /// Are we in the root config?
    root: bool,
}

impl PragmaInvalidScope {
    const fn scopes(&self) -> &'static [&'static str] {
        if self.root {
            &["line", "file", "config"]
        } else {
            &["line", "file"]
        }
    }
}

impl Code for PragmaInvalidScope {
    fn ident(&self) -> &'static str {
        "PE20"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("unknown #pragma scope `{}`", self.token.symbol(),)
    }

    fn label_message(&self) -> String {
        "unknown #pragma scope".to_string()
    }

    fn help(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), self.scopes());
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

    fn note(&self) -> Option<String> {
        Some(format!(
            "valid scopes are: {}",
            self.scopes()
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

impl PragmaInvalidScope {
    pub const fn new(token: Box<Token>, root: bool) -> Self {
        Self { token, root }
    }

    pub fn code(token: Token, root: bool) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token), root)))
    }
}
