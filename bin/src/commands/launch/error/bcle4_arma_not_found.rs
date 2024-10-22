use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct ArmaNotFound;

impl Code for ArmaNotFound {
    fn ident(&self) -> &'static str {
        "BCLE5"
    }

    fn message(&self) -> String {
        "Arma 3 not found.".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("Install Arma 3 via Steam, and run it at least once.".to_owned())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl ArmaNotFound {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
