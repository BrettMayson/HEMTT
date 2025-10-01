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

crate::analyze::lint!(LintS13Undefined);

impl Lint<LintData> for LintS13Undefined {
    fn ident(&self) -> &'static str {
        "undefined"
    }
    fn sort(&self) -> u32 {
        130
    }
    fn description(&self) -> &'static str {
        "Undefined Variable"
    }
    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **check_orphan_code**: Checks for undefined variables in orphan code blocks (code that does not seem to be directly called). This may lead to false positives. Default: false

```toml
[lints.sqf.undefined]
enabled = true
options.check_orphan_code = true
```

### Example

**Incorrect**
```sqf
systemChat _neverDefined;
```

### Explanation

Checks that variables are defined.

### Ignoring False Positives

If a variable is coming from a higher scope, you can add a #pragma to ignore the warning for specific variables.

```sqf
#pragma hemtt ignore_variables ["_fromUpper"]
_fromUpper pushBack ["newItem"];
```"#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help()
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
        let check_orphan_code =
            if let Some(toml::Value::Boolean(b)) = config.option("check_orphan_code") {
                *b
            } else {
                false
            };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::Undefined(var, range, is_orphan_scope) = issue {
                let error_hint = if *is_orphan_scope {
                    if !check_orphan_code {
                        continue;
                    }
                    Some("From Orphan Code - may not be a problem".to_owned())
                } else {
                    None
                };
                errors.push(Arc::new(CodeS13Undefined::new(
                    range.to_owned(),
                    var.to_owned(),
                    error_hint,
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS13Undefined {
    span: Range<usize>,
    variable: String,
    error_hint: Option<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS13Undefined {
    fn ident(&self) -> &'static str {
        "L-S13"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#undefined")
    }
    fn message(&self) -> String {
        format!("Undefined variable `{}`", self.variable)
    }
    fn label_message(&self) -> String {
        String::from("undefined variable")
    }
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

impl CodeS13Undefined {
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
