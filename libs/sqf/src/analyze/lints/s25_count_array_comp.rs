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
        "Count Array Comp"
    }
    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
count [] == 0
```

### Explanation

Checks for unoptimized `count array` checks."
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(true)
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
            if let Issue::CountArrayComparison(equal_zero, range) = issue {
                errors.push(Arc::new(CodeS25CountArrayComp::new(
                    range.to_owned(),
                    equal_zero.to_owned(),
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
    /// Top message
    fn message(&self) -> String {
        format!("count array comparison")
    }
    /// Under ^^^span hint
    fn label_message(&self) -> String {
        String::new()
    }
    /// bottom note
    fn note(&self) -> Option<String> {
        if self.equal_zero {
            Some("use `isEqualTo []`".into())
        } else {
            Some("use `isNotEqualTo []`".into())
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
        severity: Severity,
        processed: &Processed,
    ) -> Self {
        Self {
            span,
            equal_zero,
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
