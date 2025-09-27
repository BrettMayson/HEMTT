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

crate::analyze::lint!(LintS25CountArrayComparison);

impl Lint<LintData> for LintS25CountArrayComparison {
    fn ident(&self) -> &'static str {
        "count_array_comp"
    }
    fn sort(&self) -> u32 {
        250
    }
    fn description(&self) -> &'static str {
        "Count Array Comparison"
    }
    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
count _myArray == 0
```
**Correct**
```sqf
_myArray isEqualTo []
```

### Explanation

Checks for unoptimized array `count` checks."
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
        let mut errors: Codes = Vec::new();
        for issue in target.issues() {
            if let Issue::CountArrayComparison(equal_zero, range, variable) = issue {
                errors.push(Arc::new(CodeS25CountArrayComp::new(
                    range.to_owned(),
                    equal_zero.to_owned(),
                    variable.to_owned(),
                    config.severity(),
                    processed,
                )));
            }
        }
        errors
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS25CountArrayComp {
    span: Range<usize>,
    equal_zero: bool,
    variable: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS25CountArrayComp {
    fn ident(&self) -> &'static str {
        "L-S25"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#count_array_comp")
    }

    fn message(&self) -> String {
        "Unoptimized `count` array comparison".into()
    }

    fn label_message(&self) -> String {
        "unoptimized comparison".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        if self.equal_zero {
            Some(format!("{} isEqualTo []", self.variable))
        } else {
            Some(format!("{} isNotEqualTo []", self.variable))
        }
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS25CountArrayComp {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        equal_zero: bool,
        variable: String,
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            equal_zero,
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
