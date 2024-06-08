use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Diagnostic};

pub struct LaunchConfigNotFound {
    config: String,
    similar: Vec<String>,
}

impl Code for LaunchConfigNotFound {
    fn ident(&self) -> &'static str {
        "BCLE6"
    }

    fn message(&self) -> String {
        format!("Launch config `{}` not found.", self.config)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl LaunchConfigNotFound {
    pub fn code(config: String, available: &[String]) -> Arc<dyn Code> {
        Arc::new(Self {
            similar: similar_values(
                &config,
                &available
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            config,
        })
    }
}
