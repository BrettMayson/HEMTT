use std::sync::Arc;

use hemtt_workspace::{reporting::{Code, Diagnostic, Label, Token}, WorkspacePath};

use crate::Error;

/// An include was not found
pub struct CircularInclude {
    /// The target that was not found
    token: Vec<Token>,
    stack: Vec<WorkspacePath>,
}

impl Code for CircularInclude {
    fn ident(&self) -> &'static str {
        "PE29"
    }

    fn message(&self) -> String {
        "circular include detected".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let first = self.token.first()?;
        let last = self.token.last()?;
        Some(
            Diagnostic::new(self.ident(), self.message()).with_label(
                Label::primary(
                    first.position().path().clone(),
                    first.position().span().start..last.position().span().end,
                )
                .with_message(self.label_message()),
            ).with_note(
                format!(
                        "include stack: {}",
                    if self.stack.len() > 1 {
                        format!(
                            "\n ┬ {}\n ╰ {}",
                            self.stack
                                .iter()
                                .take(self.stack.len() - 1)
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<String>>()
                                .join("\n ├ ")
                                .as_str(),
                            self.stack.last().map(std::string::ToString::to_string).unwrap_or_default()
                        )
                    } else {
                        self.stack.first().map(std::string::ToString::to_string).unwrap_or_default()
                    }
                )
            )
        )
    }
}

impl CircularInclude {
    #[must_use]
    pub fn new(token: Vec<Arc<Token>>, stack: Vec<WorkspacePath>) -> Self {
        Self {
            token: token.into_iter().map(|t| t.as_ref().clone()).collect(),
            stack,
        }
    }

    #[must_use]
    pub fn code(token: Vec<Arc<Token>>, stack: Vec<WorkspacePath>) -> Error {
        Error::Code(Arc::new(Self::new(token, stack)))
    }
}
