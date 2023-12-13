use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

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

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, None))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl ScriptFatal {
    pub fn code(script: String) -> Arc<dyn Code> {
        Arc::new(Self { script })
    }
}
