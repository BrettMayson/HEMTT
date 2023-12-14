use std::sync::Arc;

use hemtt_common::reporting::{simple, Code};

pub struct LaunchConfigNotFound {
    config: String,
}

impl Code for LaunchConfigNotFound {
    fn ident(&self) -> &'static str {
        "BCLE6"
    }

    fn message(&self) -> String {
        format!("Launch config `{}` not found.", self.config)
    }

    fn report(&self) -> Option<String> {
        Some(simple(self, ariadne::ReportKind::Error, self.help()))
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        vec![]
    }
}

impl LaunchConfigNotFound {
    pub fn code(config: String) -> Arc<dyn Code> {
        Arc::new(Self { config })
    }
}
