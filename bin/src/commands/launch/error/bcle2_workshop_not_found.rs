use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct WorkshopNotFound;

impl Code for WorkshopNotFound {
    fn ident(&self) -> &'static str {
        "BCLE2"
    }

    fn message(&self) -> String {
        "Arma 3 workshop not found.".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl WorkshopNotFound {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
