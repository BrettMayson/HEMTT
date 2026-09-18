use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct InvalidMaterialTexturePath {
    p3d: String,
    paths: Vec<String>,
}

impl Code for InvalidMaterialTexturePath {
    fn ident(&self) -> &'static str {
        "BBE8"
    }

    fn message(&self) -> String {
        format!(
            "{} has {} material/texture path{} starting with backslash:\n  {}",
            self.p3d,
            self.paths.len(),
            if self.paths.len() == 1 { "" } else { "s" },
            self.paths.join("\n  ")
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl InvalidMaterialTexturePath {
    pub fn code(p3d: String, paths: Vec<String>) -> Arc<dyn Code> {
        Arc::new(Self { p3d, paths })
    }
}
