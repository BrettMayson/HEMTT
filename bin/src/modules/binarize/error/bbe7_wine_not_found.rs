use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Severity};

pub struct WineNotFound;

impl Code for WineNotFound {
    fn ident(&self) -> &'static str {
        "BBE7"
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn message(&self) -> String {
        String::from("`wine64` not found in PATH.")
    }

    fn note(&self) -> Option<String> {
        Some(String::from(
            "When specifying tools on Linux, make sure `wine64` is in your PATH.",
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl WineNotFound {
    #[allow(dead_code)] // only used on non-windows platforms
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}
