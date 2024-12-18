use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct ToolsNotFound;

impl Code for ToolsNotFound {
    fn ident(&self) -> &'static str {
        "BCPE1"
    }

    fn message(&self) -> String {
        String::from("Arma 3 Tools not found in registry.")
    }

    fn help(&self) -> Option<String> {
        Some(String::from(
            "Install Arma 3 Tools from Steam and run them at least once.",
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl ToolsNotFound {
    #[allow(dead_code)] // used in windows only
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}
