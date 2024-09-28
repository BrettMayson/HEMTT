use crate::{
    analyze::{
        inspector::{self, Issue, VarSource},
        SqfLintData,
    },
    Statements,
};
use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};
use std::{ops::Range, sync::Arc};
use tracing::trace;

crate::lint!(LintS12Inspector);

impl Lint<SqfLintData> for LintS12Inspector {
    fn ident(&self) -> &str {
        "inspector"
    }

    fn sort(&self) -> u32 {
        120
    }

    fn description(&self) -> &str {
        "Checks for code usage"
    }

    fn documentation(&self) -> &str {
r"### Configuration
- **check_invalid_args**: [default: true] check_invalid_args (e.g. `x setFuel true`)
- **check_child_scripts**: [default: false] Checks oprhaned scripts. 
    Assumes un-called code will run in another scope (can cause false positives)
    e.g. `private _var = 5; [{_var}] call CBA_fnc_addPerFrameEventHandler;`
- **check_undefined**: [default: true] Checks local vars that are not defined
- **check_not_private**: [default: true] Checks local vars that are not `private`
- **check_unused**: [default: false] Checks local vars that are never used
- **check_shadow**: [default: false] Checks local vars that are shaddowed

```toml
[lints.sqf.inspector]
options.check_invalid_args = true
options.check_child_scripts = true
options.check_undefined = true
options.check_not_private = true
options.check_unused = true
options.check_shadow = true
```"
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
    #[allow(clippy::too_many_lines)]
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &hemtt_common::config::LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Statements,
        data: &SqfLintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        if !target.top_level {
            // we only want to handle full files, not all the sub-statements
            return Vec::new();
        };
        let (_addon, database) = data;

        let check_invalid_args =
            if let Some(toml::Value::Boolean(b)) = config.option("check_invalid_args") {
                *b
            } else {
                true
            };
        let check_child_scripts =
                if let Some(toml::Value::Boolean(b)) = config.option("check_child_scripts") {
                    *b
                } else {
                    true // can cause false positives
                };
        let check_undefined =
            if let Some(toml::Value::Boolean(b)) = config.option("check_undefined") {
                *b
            } else {
                true
            };
        let check_not_private = if let Some(toml::Value::Boolean(b)) =
            config.option("check_not_private")
        {
            *b
        } else {
            true
        };
        let check_unused =
            if let Some(toml::Value::Boolean(b)) = config.option("check_unused") {
                *b
            } else {
                true
            };
        let check_shadow =
            if let Some(toml::Value::Boolean(b)) = config.option("check_shadow") {
                *b
            } else {
                true
            };

        let mut errors: Codes = Vec::new();
        let issues = inspector::run_processed(target, processed, database, check_child_scripts);
        trace!("issues {}", issues.len());

        for issue in issues {
            match issue {
                Issue::InvalidArgs(cmd, range) => {
                    if check_invalid_args {
                        errors.push(Arc::new(CodeS12Inspector::new(
                            range,
                            format!("Bad Args: {cmd}"),
                            None,
                            config.severity(),
                            processed,
                        )));
                    }
                }
                Issue::Undefined(var, range, is_child) => {
                    if check_undefined {
                        let error_hint = if is_child {Some("From Child Code - may not be a problem".to_owned())} else {None};
                        errors.push(Arc::new(CodeS12Inspector::new(
                            range,
                            format!("Undefined: {var}"),
                            error_hint,
                            config.severity(),
                            processed,
                        )));
                    }
                }
                Issue::NotPrivate(var, range) => {
                    if check_not_private {
                        errors.push(Arc::new(CodeS12Inspector::new(
                            range,
                            format!("NotPrivate: {var}"),
                            None,
                            config.severity(),
                            processed,
                        )));
                    }
                }
                Issue::Unused(var, source) => {
                    let VarSource::Assignment(range) = source else { continue };
                    if check_unused {
                        errors.push(Arc::new(CodeS12Inspector::new(
                            range,
                            format!("Unused: {var}"),
                            None,
                            config.severity(),
                            processed,
                        )));
                    }
                }
                Issue::Shadowed(var, range) => {
                    if check_shadow {
                        errors.push(Arc::new(CodeS12Inspector::new(
                            range,
                            format!("Shadowed: {var}"),
                            None,
                            config.severity(),
                            processed,
                        )));
                    }
                }
            };
        }

        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS12Inspector {
    span: Range<usize>,
    error_type: String,
    error_hint: Option<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS12Inspector {
    fn ident(&self) -> &'static str {
        "L-S12"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#inspector")
    }
    /// Top message
    fn message(&self) -> String {
        format!("Inspector - {}", self.error_type)
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

impl CodeS12Inspector {
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
            error_type,
            error_hint,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
