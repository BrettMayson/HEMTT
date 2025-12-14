
use std::sync::Arc;

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{Statements, analyze::{LintData, inspector::{InvalidArgs, Issue}}};

crate::analyze::lint!(LintS22ThisCall);

impl Lint<LintData> for LintS22ThisCall {
    fn ident(&self) -> &'static str {
        "invalid_return_type"
    }

    fn sort(&self) -> u32 {
        340
    }

    fn description(&self) -> &'static str {
        "Checks for invalid return types from functions (requires function header)"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
/*
 * Arguments:
 * None
 *
 * Return Value:
 * X <BOOLEAN>
 */

1234 // Not a BOOLEAN
```

### Explanation

"
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
            if let Issue::InvalidReturnType { variant } = issue {
                errors.push(Arc::new(CodeS34InvalidReturnType::new(
                    variant.to_owned(),
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS34InvalidReturnType {
    variant: InvalidArgs,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS34InvalidReturnType {
    fn ident(&self) -> &'static str {
        "L-S34"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#invalid_return_type")
    }
    fn message(&self) -> String {
        self.variant.message("")
    }
    fn label_message(&self) -> String {
        self.variant.label_message()
    }
    fn note(&self) -> Option<String> {
        Some(self.variant.note())
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS34InvalidReturnType {
    #[must_use]
    pub fn new(
        variant: InvalidArgs,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            variant,
            severity,
            diagnostic: None,
        }        .generate_processed(processed)
    }
    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.variant.span(), processed);
        self
    }
}
