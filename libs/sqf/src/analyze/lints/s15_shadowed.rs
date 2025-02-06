use crate::{
    analyze::{inspector::Issue, LintData},
    Statements,
};
use hemtt_common::config::{LintConfig, LintEnabled};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::{ops::Range, sync::Arc};

crate::analyze::lint!(LintS15Shadowed);

impl Lint<LintData> for LintS15Shadowed {
    fn ident(&self) -> &'static str {
        "shadowed"
    }
    fn sort(&self) -> u32 {
        150
    }
    fn description(&self) -> &'static str {
        "Shadowed Var"
    }
    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
private _z = 5;
private _z = 5;
```

### Explanation

Checks for variables being shadowed."
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(LintEnabled::Pedantic)
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
        target: &Statements,
        _data: &LintData,
    ) -> hemtt_workspace::reporting::Codes {
        if target.issues().is_empty() {
            return Vec::new();
        };
        let Some(processed) = processed else {
            return Vec::new();
        };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::Shadowed(var, range) = issue {
                errors.push(Arc::new(CodeS15Shadowed::new(
                    range.to_owned(),
                    var.to_owned(),
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
pub struct CodeS15Shadowed {
    span: Range<usize>,
    token_name: String,
    error_hint: Option<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS15Shadowed {
    fn ident(&self) -> &'static str {
        "L-S15"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#shadowed")
    }
    /// Top message
    fn message(&self) -> String {
        format!("Shadowed Var - {}", self.token_name)
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

impl CodeS15Shadowed {
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
