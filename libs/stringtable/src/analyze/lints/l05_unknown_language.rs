use std::sync::Arc;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Severity},
};

use crate::{analyze::LintData, Project};

crate::analyze::lint!(LintL05UnknownLanguage);

impl Lint<LintData> for LintL05UnknownLanguage {
    fn ident(&self) -> &'static str {
        "unknown_language"
    }
    fn sort(&self) -> u32 {
        50
    }
    fn description(&self) -> &'static str {
        "Checks for unknown languages"
    }
    fn documentation(&self) -> &'static str {
        "Checks for unknown languages in localization tags. Unknown languages will not be binnerized."
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

pub struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Project;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Project,
        _data: &LintData,
    ) -> Codes {
        let Err((path, unknown_info)) = target.check_unknown_languages() else {
            return vec![];
        };

        vec![Arc::new(CodeStringtableUnknownLanguage::new(
            path,
            unknown_info,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeStringtableUnknownLanguage {
    path: String,
    unknown_info: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeStringtableUnknownLanguage {
    fn ident(&self) -> &'static str {
        "L-L05"
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!(
            "Stringtable at `{}` has unknown localization tag",
            self.path
        )
    }
    fn note(&self) -> Option<String> {
        Some(self.unknown_info.clone())
    }
    fn help(&self) -> Option<String> {
        Some("Unknown languages will not be binerized".to_string())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeStringtableUnknownLanguage {
    #[must_use]
    pub fn new(
        path: String,
        unknown_info: String,
        severity: Severity,
    ) -> Self {
        Self {
            path,
            unknown_info,
            severity,
            diagnostic: None,
        }
        .generate_processed()
    }

    fn generate_processed(mut self) -> Self {
        self.diagnostic = Some(Diagnostic::from_code(&self));
        self
    }
}
