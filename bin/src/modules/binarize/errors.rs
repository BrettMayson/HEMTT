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
        Some(simple(self, ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}
impl PlatformNotSupported {
    pub fn code() -> Arc<dyn Code> {
        Arc::new(Self)
    }
}

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
