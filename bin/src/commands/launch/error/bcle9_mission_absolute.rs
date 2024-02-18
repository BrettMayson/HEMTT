use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct MissionAbsolutePath {
    reason: String,
}

impl Code for MissionAbsolutePath {
    fn ident(&self) -> &'static str {
        "BCLE9"
    }

    fn message(&self) -> String {
        "Missions can not be absolute paths.".to_string()
    }

    fn note(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl MissionAbsolutePath {
    pub fn code(reason: String) -> Arc<dyn Code> {
        Arc::new(Self { reason })
    }
}
