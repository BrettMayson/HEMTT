use std::sync::Arc;

use hemtt_common::reporting::{Code, Diagnostic};

pub struct MissingTextures {
    p3d: String,
    textures: Vec<String>,
}
impl Code for MissingTextures {
    fn ident(&self) -> &'static str {
        "BBE4"
    }

    fn message(&self) -> String {
        format!(
            "{} is missing {} texture{}:\n  {}",
            self.p3d,
            self.textures.len(),
            if self.textures.len() == 1 { "" } else { "s" },
            self.textures.join("\n  ")
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl MissingTextures {
    pub fn code(p3d: String, textures: Vec<String>) -> Arc<dyn Code> {
        Arc::new(Self {
            p3d,
            textures: textures.into_iter().map(|t| t.replace('\\', "/")).collect(),
        })
    }
}
