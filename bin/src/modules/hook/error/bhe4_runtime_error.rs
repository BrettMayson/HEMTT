use std::{path::PathBuf, sync::Arc};

use ariadne::{Label, Report, Source};
use hemtt_common::reporting::{simple, Code};
use rhai::{EvalAltResult, Position};

use super::get_offset;

pub struct RuntimeError {
    script: String,
    error: String,
    location: Position,
}

impl Code for RuntimeError {
    fn ident(&self) -> &'static str {
        "BHE3"
    }

    fn message(&self) -> String {
        format!("Script {} failed to parse", self.script)
    }

    fn report(&self) -> Option<String> {
        if self.location.position().is_none() {
            return Some(simple(self, ariadne::ReportKind::Error, None));
        }
        let content = std::fs::read_to_string(
            PathBuf::from("./hemtt/scripts/")
                .with_file_name(&self.script)
                .with_extension("rhai"),
        )
        .expect("failed to read script from error");
        let offset = get_offset(&self.script, self.location);
        let mut out = Vec::new();
        Report::build(ariadne::ReportKind::Error, self.script.as_str(), offset)
            .with_label(
                Label::new((self.script.as_str(), offset..offset))
                    .with_message(self.error.to_string()),
            )
            .finish()
            .write_for_stdout((self.script.as_str(), Source::from(content)), &mut out)
            .unwrap();
        Some(String::from_utf8(out).unwrap())
    }

    fn ci(&self) -> Vec<hemtt_common::reporting::Annotation> {
        Vec::new()
    }
}

impl RuntimeError {
    pub fn code(script: String, error: &EvalAltResult) -> Arc<dyn Code> {
        Arc::new(Self {
            script,
            error: error.to_string(),
            location: error.position(),
        })
    }
}
