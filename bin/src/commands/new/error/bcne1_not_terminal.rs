use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

pub struct TerminalNotInput;

impl Code for TerminalNotInput {
    fn ident(&self) -> &'static str {
        "BCNE1"
    }

    fn message(&self) -> String {
        "Terminal is not a TTY.".to_string()
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl TerminalNotInput {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
