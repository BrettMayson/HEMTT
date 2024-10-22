use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic, Severity};

pub struct ToolsNotFound {
    severity: Severity,
}

impl Code for ToolsNotFound {
    fn ident(&self) -> &'static str {
        "BBW1"
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("Arma 3 Tools not found.")
    }

    fn help(&self) -> Option<String> {
        if cfg!(windows) {
            Some(String::from(
                "Install Arma 3 Tools from Steam and run them at least once.",
            ))
        } else {
            Some(String::from(
                "Install Arma 3 Tools, and ensure either `wine64` or Proton Sniper is installed.",
            ))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl ToolsNotFound {
    #[allow(dead_code)] // used in windows only
    pub fn code(severity: Severity) -> Arc<dyn Code> {
        Arc::new(Self { severity })
    }
}
