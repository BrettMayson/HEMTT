use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct LaunchConfigCliOptionsNotFound {
    cli_options: Vec<String>
}

impl Code for LaunchConfigCliOptionsNotFound {
    fn ident(&self) -> &'static str {
        "BCLE10"
    }

    fn message(&self) -> String {
        format!("Launch config has one of the following cli_options that is not valid: {:?}", self.cli_options)
    }

    fn note(&self) -> Option<String> {
        Some("Make sure the cli_options in the config file are valid cli arguments for launch".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl LaunchConfigCliOptionsNotFound {
    pub fn code(cli_options: Vec<String>) -> Arc<dyn Code> {
        Arc::new(Self { cli_options })
    }
}
