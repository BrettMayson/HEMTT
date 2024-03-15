use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

#[allow(unused)]
/// An `#else` [`IfState`] was found after another `#else`
///
/// ```cpp
/// #if 1
/// #else
/// #else
/// #endif
/// ```
pub struct DoubleElse {
    /// The [`Token`] of the new `#else`
    token: Box<Token>,
    /// The [`Token`] of the previous `#else`
    previous: Box<Token>,
    /// The [`Token`] of the `#if` that this `#else` is in
    if_token: Box<Token>,
}

impl Code for DoubleElse {
    fn ident(&self) -> &'static str {
        "PE17"
    }

    fn token(&self) -> Option<&Token> {
        Some(&self.token)
    }

    fn message(&self) -> String {
        "multiple `#else` directives found in a single `#if`".to_string()
    }

    fn label_message(&self) -> String {
        "second `#else` directive".to_string()
    }

    fn expand_diagnostic(&self, diag: Diagnostic) -> Diagnostic {
        diag.with_label(
            Label::secondary(
                self.previous.position().path().clone(),
                self.previous.position().span().start..self.previous.position().span().end,
            )
            .with_message("first `#else` directive"),
        )
        .with_label(
            Label::secondary(
                self.if_token.position().path().clone(),
                self.if_token.position().span().start..self.if_token.position().span().end,
            )
            .with_message("`#if` directive"),
        )
    }
}

impl DoubleElse {
    pub fn new(token: Box<Token>, previous: Box<Token>, if_token: Box<Token>) -> Self {
        Self {
            token,
            previous,
            if_token,
        }
    }

    pub fn code(token: Token, previous: Token, if_token: Token) -> Error {
        Error::Code(Arc::new(Self::new(
            Box::new(token),
            Box::new(previous),
            Box::new(if_token),
        )))
    }
}
