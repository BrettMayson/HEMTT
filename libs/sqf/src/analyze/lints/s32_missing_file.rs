use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner}, missing::check_is_missing_file, reporting::{Code, Codes, Diagnostic, Processed, Severity}
};

use crate::{analyze::LintData, Expression};

crate::analyze::lint!(LintS30ConfigOf);

impl Lint<LintData> for LintS30ConfigOf {
    fn ident(&self) -> &'static str {
        "missing_file"
    }
    fn sort(&self) -> u32 {
        320
    }
    fn description(&self) -> &'static str {
        "Checks for missing files referenced in sqf"
    }
    fn documentation(&self) -> &'static str {
        "### Explanation

Files should exists
"
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(project) = project else {
            return Vec::new();
        };
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::String(target_str, span, _) = target else {
            return Vec::new();
        };
        if !check_is_missing_file(target_str, project, processed) {
            return Vec::new();
        }
        let span = span.start + 1..span.end - 1;
        vec![Arc::new(CodeS32MissingFile::new(
            target_str.to_string(),
            span,
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS32MissingFile {
    span: Range<usize>,
    path: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS32MissingFile {
    fn ident(&self) -> &'static str {
        "L-C32"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#file_missing")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "File Missing".to_string()
    }
    fn label_message(&self) -> String {
        "missing".to_string()
    }
    fn note(&self) -> Option<String> {
        Some(format!("file '{}' was not found in project", self.path))
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS32MissingFile {
    #[must_use]
    pub fn new(
        path: String,
        span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            path,
            span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
