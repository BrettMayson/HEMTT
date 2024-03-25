use std::{rc::Rc, sync::Arc};

use hemtt_workspace::reporting::{Code, Diagnostic, Label, Token};

use crate::Error;

/// An include was not found
pub struct IncludeNotFound {
    /// The target that was not found
    token: Vec<Token>,
}

impl Code for IncludeNotFound {
    fn ident(&self) -> &'static str {
        "PE12"
    }

    fn message(&self) -> String {
        "include not found".to_string()
    }

    fn label_message(&self) -> String {
        "not found".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        // TODO look for files with a similar name
        let first = self.token.first()?;
        let last = self.token.last()?;
        Some(
            Diagnostic::new(self.ident(), self.message()).with_label(
                Label::primary(
                    first.position().path().clone(),
                    first.position().span().start..last.position().span().end,
                )
                .with_message(self.label_message()),
            ),
        )
    }
}

impl IncludeNotFound {
    pub fn new(token: Vec<Rc<Token>>) -> Self {
        Self {
            token: token.into_iter().map(|t| t.as_ref().clone()).collect(),
        }
    }

    pub fn code(token: Vec<Rc<Token>>) -> Error {
        Error::Code(Arc::new(Self::new(token)))
    }
}
