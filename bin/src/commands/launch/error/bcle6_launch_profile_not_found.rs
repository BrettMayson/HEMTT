use std::sync::Arc;

use hemtt_common::similar_values;
use hemtt_workspace::reporting::{Code, Diagnostic};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LaunchSource {
    Global,
    Project,
    CDLC,
}

pub struct LaunchProfileNotFound {
    source: LaunchSource,
    profile: String,
    similar: Vec<String>,
}

impl Code for LaunchProfileNotFound {
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
            format!("Global launch profile `{}` not found.", self.profile)
        } else if self.source == LaunchSource::CDLC {
            format!("CDLC `{}` not found.", self.profile)
        } else {
            format!("Launch profile `{}` not found.", self.profile)
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

impl LaunchProfileNotFound {
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
            profile: config,
        })
    }
}
