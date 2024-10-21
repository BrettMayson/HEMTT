use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::SqfLintData, Expression, UnaryCommand};

crate::lint!(LintS19ExtraNot);

impl Lint<SqfLintData> for LintS19ExtraNot {
    fn ident(&self) -> &str {
        "extra_not"
    }

    fn sort(&self) -> u32 {
        190
    }

    fn description(&self) -> &str {
        "Checks for extra not before a comparison"
    }

    fn documentation(&self) -> &str {
        r"### Example

**Incorrect**
```sqf
! (5 isEqualTo 6)
```
**Correct**
```sqf
(5 isNotEqualTo 6)
```
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> Codes {
        const COMPARISON_CMDS: &[&str] = &[
            "==",
            "!=",
            "isEqualTo",
            "isNotEqualTo",
            "<",
            "<=",
            ">",
            ">=",
        ];
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Not, rhs, range) = target else {
            return Vec::new();
        };
        let Expression::BinaryCommand(ref last_cmd, _, _, _) = **rhs else {
            return Vec::new();
        };
        if !COMPARISON_CMDS.contains(&last_cmd.as_str()) {
            return Vec::new();
        }

        vec![Arc::new(Code19ExtraNot::new(
            range.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Code19ExtraNot {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code19ExtraNot {
    fn ident(&self) -> &'static str {
        "L-S19"
    }
    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#extra_not")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "Unneeded Not".to_string()
    }
    // fn label_message(&self) -> String {
    //     "".to_string()
    // }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code19ExtraNot {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
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
