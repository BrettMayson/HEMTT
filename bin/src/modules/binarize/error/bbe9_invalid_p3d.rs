use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct InvalidP3D {
    file: String,
    reason: String,
}
impl Code for InvalidP3D {
    fn ident(&self) -> &'static str {
        "BBE9"
    }

    fn message(&self) -> String {
        format!("Unable to read P3D {}", self.file)
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn help(&self) -> Option<String> {
        Some(
            "The file may be corrupt or truncated, try saving it again from Object Builder"
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl InvalidP3D {
    pub fn code(file: String, reason: String) -> Arc<dyn Code> {
        Arc::new(Self { file, reason })
    }
}
