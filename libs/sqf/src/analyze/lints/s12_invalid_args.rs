use crate::{
    analyze::{inspector::Issue, SqfLintData},
    Statements,
};
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::{ops::Range, sync::Arc};

crate::analyze::lint!(LintS12InvalidArgs);

impl Lint<SqfLintData> for LintS12InvalidArgs {
    fn ident(&self) -> &'static str {
        "invalid_args"
    }
    fn sort(&self) -> u32 {
        120
    }
    fn description(&self) -> &'static str {
        "Invalid Args"
    }
    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
(vehicle player) setFuel true; // bad args: takes number 0-1
```

### Explanation

Checks correct syntax usage."
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }
    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

pub struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Statements;
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Statements,
        _data: &SqfLintData,
    ) -> hemtt_workspace::reporting::Codes {
        if target.issues().is_empty() {
            return Vec::new();
        };
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::InvalidArgs(cmd, range) = issue {
                errors.push(Arc::new(CodeS12InvalidArgs::new(
                    range.to_owned(),
                    cmd.to_owned(),
                    None,
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS12InvalidArgs {
    span: Range<usize>,
    token_name: String,
    error_hint: Option<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS12InvalidArgs {
    fn ident(&self) -> &'static str {
        "L-S12"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#invalid_args")
    }
    /// Top message
    fn message(&self) -> String {
        format!("Invalid Args - {}", self.token_name)
    }
    /// Under ^^^span hint
    fn label_message(&self) -> String {
        String::new()
    }
    /// bottom note
    fn note(&self) -> Option<String> {
        self.error_hint.clone()
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS12InvalidArgs {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        error_type: String,
        error_hint: Option<String>,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            token_name: error_type,
            error_hint,
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
