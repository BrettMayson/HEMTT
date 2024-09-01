use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{diagnostic::Yellow, Code, Diagnostic, Processed},
};

use crate::Property;

#[allow(clippy::module_name_repetitions)]
pub struct LintC08MissingSemicolon;

impl Lint for LintC08MissingSemicolon {
    fn ident(&self) -> &str {
        "missing_semicolon"
    }

    fn description(&self) -> &str {
        "Missing semicolon"
    }

    fn documentation(&self) -> &str {
        "The property is missing a semicolon"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;

impl LintRunner for Runner {
    type Target = crate::Property;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &crate::Property,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        if let Property::MissingSemicolon(_, span) = target {
            vec![Arc::new(Code08MissingSemicolon::new(
                span.clone(),
                processed,
            ))]
        } else {
            vec![]
        }
    }
}

pub struct Code08MissingSemicolon {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code08MissingSemicolon {
    fn ident(&self) -> &'static str {
        "L-C08"
    }

    fn message(&self) -> String {
        "property is missing a semicolon".to_string()
    }

    fn label_message(&self) -> String {
        "missing semicolon".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "add a semicolon {} to the end of the property",
            Yellow.paint(";")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code08MissingSemicolon {
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.as_str()[self.span.clone()];
        let possible_end = self.span.start
            + haystack
                .find('\n')
                .unwrap_or_else(|| haystack.rfind(|c: char| c != ' ' && c != '}').unwrap_or(0) + 1);
        self.diagnostic =
            Diagnostic::new_for_processed(&self, possible_end..possible_end, processed);
        self
    }
}
