use std::sync::Arc;

use hemtt_workspace::{
    reporting::{Code, Diagnostic, Label},
    WorkspacePath,
};
use rhai::{ParseErrorType, Position};

use super::get_offset;

pub struct RhaiParseError {
    script: WorkspacePath,
    error: Box<ParseErrorType>,
    location: Position,
}

impl Code for RhaiParseError {
    fn ident(&self) -> &'static str {
        "BHE3"
    }

    fn message(&self) -> String {
        format!("Script {} failed to parse", self.script)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let content = self.script.read_to_string().ok()?;
        Some(
            Diagnostic::from_code(self).with_label(
                Label::primary(
                    self.script.clone(),
                    get_offset(&content, self.location)..get_offset(&content, self.location),
                )
                .with_message(format!("{}", self.error)),
            ),
        )
    }
}

impl RhaiParseError {
    pub fn code(
        script: WorkspacePath,
        error: Box<ParseErrorType>,
        location: Position,
    ) -> Arc<dyn Code> {
        Arc::new(Self {
            script,
            error,
            location,
        })
    }
}
