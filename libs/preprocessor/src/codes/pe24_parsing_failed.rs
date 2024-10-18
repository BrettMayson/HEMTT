use std::sync::Arc;

use hemtt_workspace::{
    position::{LineCol, Position},
    reporting::{Code, Diagnostic, Label},
    WorkspacePath,
};
use pest::error::InputLocation;

use crate::{parse::Rule, Error};

#[allow(unused)]
/// Unexpected token
pub struct ParsingFailed {
    error: pest::error::Error<Rule>,
    position: Position,
    report: Option<String>,
}

impl Code for ParsingFailed {
    fn ident(&self) -> &'static str {
        "PE24"
    }

    fn message(&self) -> String {
        "failed to parse".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(
            Diagnostic::new(self.ident(), "failed to parse").with_label(
                Label::primary(self.position.path().clone(), self.position.span())
                    .with_message("failed to parse"),
            ),
        )
    }
}

impl ParsingFailed {
    #[must_use]
    pub fn new(error: pest::error::Error<Rule>, file: WorkspacePath) -> Self {
        let content = file.read_to_string().unwrap_or_default();
        let span = match &error.location {
            InputLocation::Pos(pos) => pos..pos,
            InputLocation::Span((start, end)) => start..end,
        };
        let start = LineCol::from_content(&content, *span.start);
        let end = LineCol::from_content(&content, *span.end);
        let position = Position::new(start, end, file);
        Self {
            error,
            position,
            report: None,
        }
    }

    #[must_use]
    pub fn code(error: pest::error::Error<Rule>, file: WorkspacePath) -> Error {
        Error::Code(Arc::new(Self::new(error, file)))
    }
}
