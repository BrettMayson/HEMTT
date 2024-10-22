use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct WorkshopModNotFound {
    id: String,
}

impl Code for WorkshopModNotFound {
    fn ident(&self) -> &'static str {
        "BCLE2"
    }

    fn link(&self) -> Option<&str> {
        Some("/commands/launch.html#workshop")
    }

    fn message(&self) -> String {
        format!("Arma 3 workshop mod `{}` not found.", self.id)
    }

    fn help(&self) -> Option<String> {
        Some(format!("HEMTT does not subscribe to mods, you must subscribe in Steam and allow it to download.\nWorkshop link: https://steamcommunity.com/sharedfiles/filedetails/?id={}", self.id))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl WorkshopModNotFound {
    pub fn code(id: String) -> Arc<dyn Code> {
        Arc::new(Self { id })
    }
}
