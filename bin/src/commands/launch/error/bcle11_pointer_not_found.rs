use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct PointerNotFound {
    id: String,
    child: String,
}

impl Code for PointerNotFound {
    fn ident(&self) -> &'static str {
        "BCLE11"
    }

    fn link(&self) -> Option<&str> {
        Some("/commands/launch.html#pointers")
    }

    fn message(&self) -> String {
        format!("Arma 3 Pointer `{}` not found", self.id)
    }

    fn note(&self) -> Option<String> {
        Some(format!("Tried loading mod `{}`", self.child))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl PointerNotFound {
    #[must_use]
    pub fn code(id: String, child: String) -> Arc<dyn Code> {
        Arc::new(Self { id, child })
    }
}
