use std::sync::Arc;

use hemtt_workspace::reporting::{diagnostic::Yellow, Code, Diagnostic, Label, Token};

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
        format!("invalid #if operator `{}`", self.tokens.iter().map(std::string::ToString::to_string).collect::<String>())
    }

    fn label_message(&self) -> String {
        "invalid operator".to_string()
    }

    fn help(&self) -> Option<String> {
        let valid = ["==", "!=", "<", ">", "<=", ">="];
        Some(format!(
            "valid operators are {}",
            Yellow.paint(valid.join(" "))
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let start = self.tokens.first()?;
        let end = self.tokens.last()?;
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
        diag: hemtt_workspace::reporting::Diagnostic,
    ) -> hemtt_workspace::reporting::Diagnostic {
        diag
    }
}

impl IfInvalidOperator {
    #[must_use]
    pub const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    #[must_use]
    pub fn code(tokens: Vec<Token>) -> Error {
        Error::Code(Arc::new(Self::new(tokens)))
    }
}
