use std::sync::Arc;

use ariadne::ReportKind;
use hemtt_common::reporting::{simple, Code};

pub struct ToolsNotFound;

impl Code for ToolsNotFound {
    fn ident(&self) -> &'static str {
        "BBE1"
    }

    fn message(&self) -> String {
        String::from("Arma 3 Tools not found in registry.")
    }

    fn help(&self) -> Option<String> {
        Some(String::from(
            "Install Arma 3 Tools from Steam and run them at least once.",
        ))
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl ToolsNotFound {
    #[allow(dead_code)] // used in windows only
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}
