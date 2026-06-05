use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct AceNotLoaded {}

impl Code for AceNotLoaded {
    fn ident(&self) -> &'static str {
        "BCLE12"
    }

    fn link(&self) -> Option<&str> {
        Some("/commands/launch.html#ace_not_loaded")
    }

    fn message(&self) -> String {
        String::from("ACE Arsenal mission selected but ACE mod not loaded")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl AceNotLoaded {
    #[must_use]
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
