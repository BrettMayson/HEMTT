use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

pub struct WorkshopNotFound;

impl Code for WorkshopNotFound {
    fn ident(&self) -> &'static str {
        "BCLE2"
    }

    fn message(&self) -> String {
        "Arma 3 workshop not found.".to_string()
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl WorkshopNotFound {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
