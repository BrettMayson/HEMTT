use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct BinarizeFailed {
    file: String,
}
impl Code for BinarizeFailed {
    fn ident(&self) -> &'static str {
        "BBE3"
    }

    fn message(&self) -> String {
        format!("No output found for {}, binarization failed.", self.file)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl BinarizeFailed {
    pub fn code(file: String) -> Arc<dyn Code> {
        Arc::new(Self { file })
    }
}
