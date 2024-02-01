use std::sync::Arc;

use ariadne::ReportKind;
use hemtt_common::reporting::{simple, Code};

pub struct PlatformNotSupported;

impl Code for PlatformNotSupported {
    fn ident(&self) -> &'static str {
        "BBE2"
    }

    fn message(&self) -> String {
        String::from("Platform not supported for binarization.")
    }

    fn help(&self) -> Option<String> {
        Some(String::from("HEMTT only supports binarization on Windows."))
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ReportKind::Warning, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl PlatformNotSupported {
    #[allow(dead_code)] // only used on non-windows platforms
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}
