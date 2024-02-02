use std::sync::Arc;

use hemtt_common::reporting::{diagnostic::Yellow, Code, Diagnostic, Label, Token};

use crate::Error;
#[allow(unused)]
/// Unexpected token
pub struct IfInvalidOperator {
    /// The [`Token`]s that were found
    tokens: Vec<Token>,
}

impl Code for IfInvalidOperator {
    fn ident(&self) -> &'static str {
        "PE15"
    }

    fn message(&self) -> String {
        "invalid #if operator".to_string()
    }

    fn help(&self) -> Option<String> {
        let valid = ["==", "!=", "<", ">", "<=", ">="];
        Some(format!(
            "valid operators are {}",
            Yellow.paint(valid.join(" "))
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let Some(start) = self.tokens.first() else {
            return None;
        };
        let Some(end) = self.tokens.last() else {
            return None;
        };
        Some(
            Diagnostic::new(self.ident(), self.message())
                .with_label(
                    Label::primary(
                        start.position().path().clone(),
                        start.position().span().start..end.position().span().end,
                    )
                    .with_message(self.label_message()),
                )
                .with_help(self.help()?),
        )
    }

    fn expand_diagnostic(
        &self,
        diag: hemtt_common::reporting::Diagnostic,
    ) -> hemtt_common::reporting::Diagnostic {
        diag
    }
}

impl IfInvalidOperator {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub fn code(tokens: Vec<Token>) -> Error {
        Error::Code(Arc::new(Self::new(tokens)))
    }
}
