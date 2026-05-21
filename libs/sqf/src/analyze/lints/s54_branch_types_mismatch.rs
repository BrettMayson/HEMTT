use crate::{
    analyze::{inspector::Issue, LintData},
    Statements,
};
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::{ops::Range, sync::Arc};

crate::analyze::lint!(LintS54BranchTypesMismatch);

impl Lint<LintData> for LintS54BranchTypesMismatch {
    fn ident(&self) -> &'static str {
        "branch_types_mismatch"
    }
    fn sort(&self) -> u32 {
        540
    }
    fn description(&self) -> &'static str {
        "Checks for branch types mismatch"
    }
    fn documentation(&self) -> &'static str {
        r#"
        ### Example

**Incorrect**
```sqf
x = if (y) then { 1 } else { "string" };
```
"#
    }

    fn default_config(&self) -> LintConfig {
        // there are legit reasons to have mismatched types
        LintConfig::warning().with_enabled(hemtt_common::config::LintEnabled::Pedantic)
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

pub struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Statements;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Statements,
        _data: &LintData,
    ) -> hemtt_workspace::reporting::Codes {
        if target.issues().is_empty() {
            return Vec::new();
        }
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::MismatchedTypes { span, command } = issue {
                errors.push(Arc::new(CodeS54BranchTypesMismatch::new(
                    span.to_owned(),
                    command.clone(),
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS54BranchTypesMismatch {
    span: Range<usize>,
    command: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS54BranchTypesMismatch {
    fn ident(&self) -> &'static str {
        "L-S54"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#branch_types_mismatch")
    }
    fn message(&self) -> String {
        format!("Branch types mismatch in command `{}`", self.command)
    }
    fn label_message(&self) -> String {
        "mismatched types".to_string()
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS54BranchTypesMismatch {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        command: String,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            command,
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
