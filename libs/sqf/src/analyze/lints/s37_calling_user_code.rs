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

crate::analyze::lint!(LintS37CallingUserCode);

impl Lint<LintData> for LintS37CallingUserCode {
    fn ident(&self) -> &'static str {
        "calling_user_code"
    }
    fn sort(&self) -> u32 {
        370
    }
    fn description(&self) -> &'static str {
        "Checks for user code possibly being called"
    }
    fn documentation(&self) -> &'static str {
        r#"
        ### Example

**Incorrect**
```sqf
setting = profileNamespace getVariable "test";
x || setting
```
**Correct**
```sqf
setting = profileNamespace getVariable "test";
x || (setting isEqualTo true)
```
"#
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
        let ignore = if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            ignore.iter().map(|v| v.as_str().expect("ignore items must be strings")).collect::<Vec<&str>>()
        } else {
            vec![]
        };
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::CallingUserCode { span, var } = issue {
                if ignore.iter().any(|s| s.eq_ignore_ascii_case(var.as_str())) {
                    continue;
                }
                errors.push(Arc::new(CodeS37CallingUserCode::new(
                    span.to_owned(),
                    var.to_owned(),
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS37CallingUserCode {
    span: Range<usize>,
    variable: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS37CallingUserCode {
    fn ident(&self) -> &'static str {
        "L-S37"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#calling_user_code")
    }
    fn message(&self) -> String {
        "Calling user code".to_string()
    }
    fn label_message(&self) -> String {
        format!("variable `{}` may be user code", self.variable)
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS37CallingUserCode {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        variable: String,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            variable,
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
