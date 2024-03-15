use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct ScriptFatal {
    script: String,
}

impl Code for ScriptFatal {
    fn ident(&self) -> &'static str {
        "BHE2"
    }

    fn message(&self) -> String {
        format!("Script {} signalled fatal", self.script)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl ScriptFatal {
    pub fn code(script: String) -> Arc<dyn Code> {
        Arc::new(Self { script })
    }
}
