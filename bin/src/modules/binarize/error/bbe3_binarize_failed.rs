use std::sync::Arc;

use ariadne::ReportKind;
use hemtt_common::reporting::{simple, Code};

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

    fn report(&self) -> Option<String> {
        Some(simple(self, ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl BinarizeFailed {
    pub fn code(file: String) -> Arc<dyn Code> {
        Arc::new(Self { file })
    }
}
