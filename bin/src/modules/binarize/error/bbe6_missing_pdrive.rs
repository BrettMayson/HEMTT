use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct MissingPDrive {}
impl Code for MissingPDrive {
    fn ident(&self) -> &'static str {
        "BBE6"
    }

    fn message(&self) -> String {
        "This project requires a P Drive but no P Drive was found nor can one be emulated from an Arma 3 installation.".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl MissingPDrive {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
