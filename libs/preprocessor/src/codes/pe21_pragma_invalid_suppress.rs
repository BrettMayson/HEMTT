use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Token};

use crate::{processor::pragma::Suppress, Error};

#[allow(unused)]
/// An unknown `#pragma hemtt suppress` code
///
/// ```cpp
/// #pragma hemtt suppress unknown
/// ```
pub struct PragmaInvalidSuppress {
    /// The [`Token`] of the code
    token: Box<Token>,
}

impl Code for PragmaInvalidSuppress {
    fn ident(&self) -> &'static str {
        "PE21"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        format!("unknown #pragma suppress `{}`", self.token.symbol(),)
    }

    fn label_message(&self) -> String {
        "unknown #pragma suppress".to_string()
    }

    fn help(&self) -> Option<String> {
        let similar = similar_values(self.token.to_string().as_str(), Suppress::as_slice());
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

impl PragmaInvalidSuppress {
    pub fn new(token: Box<Token>) -> Self {
        Self { token }
    }

    pub fn code(token: Token) -> Error {
        Error::Code(Arc::new(Self::new(Box::new(token))))
    }
}
