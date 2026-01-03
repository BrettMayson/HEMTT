use crate::{
    analyze::{
        inspector::{Issue, VarSource},
        LintData,
    },
    Statements,
};
use hemtt_common::config::{LintConfig, LintEnabled};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::{ops::Range, sync::Arc};

crate::analyze::lint!(LintS14Unused);

impl Lint<LintData> for LintS14Unused {
    fn ident(&self) -> &'static str {
        "unused"
    }
    fn sort(&self) -> u32 {
        120
    }
    fn description(&self) -> &'static str {
        "Unused Var"
    }
    fn documentation(&self) -> &'static str {
        r#"### Configuration

- **check_params**: Checks for unused variables in `params` arrays. Default: false

```toml
[lints.sqf.unused]
enabled = true
options.check_params = true
```

### Example

**Incorrect**
```sqf
private _z = 5; // and never used
```

### Explanation

Checks for variables that are never used.

### Ignoring False Positives

You can add a #pragma to ignore the warning for specific variables.

```sqf
#pragma hemtt ignore_variables ["_z"]
private _z = 5; // and never used
```"#
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

        let check_params = if let Some(toml::Value::Boolean(b)) = config.option("check_params") {
            *b
        } else {
            false
        };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::Unused(var, source, overwritten) = issue {
                match source {
                    VarSource::Assignment(_, _) => {}
                    VarSource::Params(_) => {
                        if !check_params {
                            continue;
                        }
                    }
                    _ => {
                        continue;
                    }
                }
                errors.push(Arc::new(CodeS14Unused::new(
                    source.get_range().unwrap_or_default(),
                    var.to_owned(),
                    *overwritten,
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS14Unused {
    span: Range<usize>,
    variable: String,
    overwritten: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS14Unused {
    fn ident(&self) -> &'static str {
        "L-S14"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#unused")
    }
    fn message(&self) -> String {
        format!("Unused variable `{}`", self.variable)
    }
    fn label_message(&self) -> String {
        String::from("unused variable")
    }
    fn note(&self) -> Option<String> {
        if self.overwritten {
            Some(String::from("variable overwritten before being used"))
        } else {
            None
        }
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS14Unused {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        variable: String,
        overwritten: bool,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            variable,
            overwritten,
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
