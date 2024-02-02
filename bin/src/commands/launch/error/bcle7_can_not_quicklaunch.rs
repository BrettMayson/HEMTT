use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct CanNotQuickLaunch {
    reason: String,
}

impl Code for CanNotQuickLaunch {
    fn ident(&self) -> &'static str {
        "BCLE7"
    }

    fn message(&self) -> String {
        "Unable to quick launch.".to_string()
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl CanNotQuickLaunch {
    pub fn code(reason: String) -> Arc<dyn Code> {
        Arc::new(Self { reason })
    }
}
