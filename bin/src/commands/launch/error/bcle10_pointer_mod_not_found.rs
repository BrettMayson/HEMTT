use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct PointerModNotFound {
    id: String,
    location: String,
}

impl Code for PointerModNotFound {
    fn ident(&self) -> &'static str {
        "BCLE10"
    }

    fn link(&self) -> Option<&str> {
        Some("/commands/launch.html#pointers")
    }

    fn message(&self) -> String {
        format!(
            "Arma 3 Pointer mod `{}` not found at `{}`",
            self.id, self.location
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl PointerModNotFound {
    #[must_use]
    pub fn code(id: String, location: String) -> Arc<dyn Code> {
        Arc::new(Self { id, location })
    }
}
