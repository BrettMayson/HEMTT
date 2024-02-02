use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct FolderExists {
    name: String,
}

impl Code for FolderExists {
    fn ident(&self) -> &'static str {
        "BCNE2"
    }

    fn message(&self) -> String {
        format!("Folder `{}` already exists.", self.name)
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl FolderExists {
    pub fn code(name: String) -> Arc<dyn Code> {
        Arc::new(Self { name })
    }
}
