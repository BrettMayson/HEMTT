use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

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
        Some(Diagnostic::from_code(self))
    }
}

impl CanNotQuickLaunch {
    #[must_use]
    pub fn code(reason: String) -> Arc<dyn Code> {
        Arc::new(Self { reason })
    }
}
