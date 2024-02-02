use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct TerminalNotInput;

impl Code for TerminalNotInput {
    fn ident(&self) -> &'static str {
        "BCNE1"
    }

    fn message(&self) -> String {
        "Terminal is not a TTY.".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl TerminalNotInput {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
