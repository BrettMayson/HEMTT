use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Diagnostic};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LaunchSource {
    Global,
    Project,
    CDLC,
}

pub struct LaunchConfigNotFound {
    source: LaunchSource,
    config: String,
    similar: Vec<String>,
}

impl Code for LaunchConfigNotFound {
    fn ident(&self) -> &'static str {
        "BCLE6"
    }

    fn link(&self) -> Option<&str> {
        if self.source == LaunchSource::Global {
            Some("/commands/launch.html#global-configuration")
        } else if self.source == LaunchSource::CDLC {
            Some("/commands/launch.html#cdlc-launch")
        } else {
            Some("/commands/launch.html#configuration")
        }
    }

    fn message(&self) -> String {
        if self.source == LaunchSource::Global {
            format!("Global launch config `{}` not found.", self.config)
        } else if self.source == LaunchSource::CDLC {
            format!("CDLC `{}` not found.", self.config)
        } else {
            format!("Launch config `{}` not found.", self.config)
        }
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::from_code(self))
    }
}

impl LaunchConfigNotFound {
    pub fn code(source: LaunchSource, config: String, available: &[String]) -> Arc<dyn Code> {
        Arc::new(Self {
            source,
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
