use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

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

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl ArmaNotFound {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self {})
    }
}
