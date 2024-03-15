use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct MissingMaterials {
    p3d: String,
    materials: Vec<String>,
}
impl Code for MissingMaterials {
    fn ident(&self) -> &'static str {
        "BBE5"
    }

    fn message(&self) -> String {
        format!(
            "{} is missing {} material{}:\n  {}",
            self.p3d,
            self.materials.len(),
            if self.materials.len() == 1 { "" } else { "s" },
            self.materials.join("\n  ")
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl MissingMaterials {
    pub fn code(p3d: String, materials: Vec<String>) -> Arc<dyn Code> {
        Arc::new(Self {
            p3d,
            materials: materials
                .into_iter()
                .map(|t| t.replace('\\', "/"))
                .collect(),
        })
    }
}
