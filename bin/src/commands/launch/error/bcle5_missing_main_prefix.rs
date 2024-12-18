use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct MissingMainPrefix;

impl Code for MissingMainPrefix {
    fn ident(&self) -> &'static str {
        "BCLE5"
    }

    fn link(&self) -> Option<&str> {
        Some("/configuration/index.html#main-prefix")
    }

    fn message(&self) -> String {
        "Missing `mainprefix` in project.toml.".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl MissingMainPrefix {
    #[must_use]
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
