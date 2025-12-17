use crate::{
    Statements, analyze::{LintData, inspector::{InvalidArgs, Issue}}
};
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::sync::Arc;

crate::analyze::lint!(LintS12InvalidArgs);

impl Lint<LintData> for LintS12InvalidArgs {
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
        r#"### Example

**Incorrect**
```sqf
(vehicle player) setFuel true; // bad args: takes number 0-1
```

### Explanation

Checks correct syntax usage.

### Ignoring False Positives

If a variable usage is complicated and causing a false positive, you can add a #pragma to ignore the warning.

```sqf
if (something) then { CouldBeNumberOrString = 5;};        // Lint will assume the variable is a number
if (otherthing) then { y = CouldBeNumberOrString + "c";}; // Throws: Invalid arguments to command `[B:+]`

// But the var could actually be set to a string from a different scope, so ignore this specific warning with:
#pragma hemtt ignore_variables ["CouldBeNumberOrString"]  // Lint will now assume the variable could be anything
```"#
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
            if let Issue::InvalidArgs { command, variant, .. } = issue {
                errors.push(Arc::new(CodeS12InvalidArgs::new(
                    command.to_owned(),
                    variant.clone(),
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
    command: String,
    variant: InvalidArgs,
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
    fn message(&self) -> String {
        self.variant.message(&self.command)
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

impl CodeS12InvalidArgs {
    #[must_use]
    pub fn new(
        command: String,
        variant: InvalidArgs,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            command,
            variant,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }
    fn generate_processed(mut self, processed: &Processed) -> Self {
        let diag = Diagnostic::from_code_processed(&self, self.variant.span(), processed);
        if let Some(mut diag) = diag {
            if let InvalidArgs::DefaultDifferentType { default: Some(default), .. } = &self.variant {
                let map = processed
                        .mapping(default.start)
                        .expect("mapping should exist");
                    let file = processed.source(map.source()).expect("source should exist");
                    diag.labels.push(
                        hemtt_workspace::reporting::Label::secondary(
                            file.0.clone(),
                            map.original_start()..map.original_start() + default.len(),
                        )
                        .with_message("expected types"),
                    );
            }
            self.diagnostic = Some(diag);
        }
        self
    }
}
