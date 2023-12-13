use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

pub struct MissingMainPrefix;

impl Code for MissingMainPrefix {
    fn ident(&self) -> &'static str {
        "BCLE5"
    }

    fn message(&self) -> String {
        "Missing `mainprefix` in project.toml.".to_string()
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl MissingMainPrefix {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
