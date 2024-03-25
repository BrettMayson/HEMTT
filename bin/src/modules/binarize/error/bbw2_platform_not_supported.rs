use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Severity};

pub struct PlatformNotSupported;

impl Code for PlatformNotSupported {
    fn ident(&self) -> &'static str {
        "BBW2"
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn message(&self) -> String {
        String::from("Platform not supported for binarization.")
    }

    fn note(&self) -> Option<String> {
        Some(String::from("HEMTT only supports binarization on Windows."))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl PlatformNotSupported {
    #[allow(dead_code)] // only used on non-windows platforms
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}
