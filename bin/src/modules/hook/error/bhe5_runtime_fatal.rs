use std::sync::Arc;

use hemtt_workspace::{
    WorkspacePath,
    reporting::{Code, Diagnostic, Label},
};
use rhai::Position;

use super::get_offset;

pub struct RuntimeFatal {
    script: WorkspacePath,
    error: String,
    location: Position,
}

impl Code for RuntimeFatal {
    fn ident(&self) -> &'static str {
        "BHE5"
    }

    fn message(&self) -> String {
        format!("Script {} intentionally failed at runtime", self.script)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        let content = self.script.read_to_string().ok()?;
        Some(
            Diagnostic::from_code(self).with_label(
                Label::primary(
                    self.script.clone(),
                    get_offset(&content, self.location)..get_offset(&content, self.location),
                )
                .with_message({
                    let mut chars = self.error.chars();
                    chars.next().map_or_else(
                        || self.error.clone(),
                        |first| first.to_lowercase().collect::<String>() + chars.as_str(),
                    )
                }),
            ),
        )
    }
}

impl RuntimeFatal {
    pub fn code(script: WorkspacePath, error: String, location: Position) -> Arc<dyn Code> {
        Arc::new(Self {
            script,
            error,
            location,
        })
    }
}
