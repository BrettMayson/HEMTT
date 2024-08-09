use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct LaunchConfigNotStartingWithDashDash {
    cli_option: String,
    launch_config: String,
}

impl Code for LaunchConfigNotStartingWithDashDash {
    fn ident(&self) -> &'static str {
        "BCLE10"
    }

    fn message(&self) -> String {
        format!(
            "The option '{}' in Launch config: '{}' does not start with '--'",
            self.cli_option, self.launch_config
        )
    }

    fn note(&self) -> Option<String> {
        Some(
            "If using arguments with optional values remember to put them on the same line like: '--optional=some_optional'"
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl LaunchConfigNotStartingWithDashDash {
    pub fn code(cli_option: String, launch_config: String) -> Arc<dyn Code> {
        Arc::new(Self {
            cli_option,
            launch_config,
        })
    }
}
