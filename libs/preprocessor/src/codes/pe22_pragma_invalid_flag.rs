use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Token};

use crate::{processor::pragma::Flag, Error};

#[allow(unused)]
/// An unknown `#pragma hemtt flag` code
///
/// ```cpp
/// #pragma hemtt flag unknown
/// ```
pub struct PragmaInvalidFlag {
    /// The [`Token`] of the code
    token: Box<Token>,
}

impl Code for PragmaInvalidFlag {
    fn ident(&self) -> &'static str {
        "PE22"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("unknown #pragma flag `{}`", self.token.symbol())
    }

    fn label_message(&self) -> String {
        "unknown #pragma flag`".to_string()
    }

    fn help(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), Flag::as_slice());
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
}

impl PragmaInvalidFlag {
    pub const fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
