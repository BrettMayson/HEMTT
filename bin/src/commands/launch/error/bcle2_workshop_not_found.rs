use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct WorkshopNotFound;

impl Code for WorkshopNotFound {
    fn ident(&self) -> &'static str {
        "BCLE2"
    }

    fn message(&self) -> String {
        "Arma 3 workshop not found.".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(
            "Run Arma 3 at least once from Steam before attempting to use `hemtt launch`."
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl WorkshopNotFound {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
