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

crate::analyze::lint!(LintS16NotPrivate);

impl Lint<LintData> for LintS16NotPrivate {
    fn ident(&self) -> &'static str {
        "not_private"
    }
    fn sort(&self) -> u32 {
        160
    }
    fn description(&self) -> &'static str {
        "Not Private Var"
    }
    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
_z = 6;
```

### Explanation

Checks local variables that are not private.

### Ignoring False Positives

If a variable is coming from a higher scope and cannot be private, you can add a #pragma to ignore the warning for specific variables.

```sqf
#pragma hemtt ignore_not_private ["_fromUpper"]
_fromUpper pushBack ["newItem"];
```
"#
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
            if let Issue::NotPrivate(var, range) = issue {
                errors.push(Arc::new(CodeS16NotPrivate::new(
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
pub struct CodeS16NotPrivate {
    span: Range<usize>,
    variable: String,
    error_hint: Option<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS16NotPrivate {
    fn ident(&self) -> &'static str {
        "L-S16"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#not_private")
    }
    fn message(&self) -> String {
        format!("Local variable `{}` is not private", self.variable)
    }
    fn label_message(&self) -> String {
        String::from("can be private")
    }
    fn note(&self) -> Option<String> {
        self.error_hint.clone()
    }
    fn suggestion(&self) -> Option<String> {
        Some(format!("private {}", self.variable))
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS16NotPrivate {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        variable: String,
        error_hint: Option<String>,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            variable,
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
