use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

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

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl FolderExists {
    pub fn code(name: String) -> Arc<dyn Code> {
        Arc::new(Self { name })
    }
}
