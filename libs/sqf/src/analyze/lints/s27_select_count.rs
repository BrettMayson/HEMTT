use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, UnaryCommand};

crate::analyze::lint!(LintS27SelectCount);

impl Lint<LintData> for LintS27SelectCount {
    fn ident(&self) -> &'static str {
        "select_count"
    }

    fn sort(&self) -> u32 {
        270
    }

    fn description(&self) -> &'static str {
        "Checks for `_array select (count _array - 1)` and suggests `_array select -1`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
_array select (count _array - 1);
```
**Correct**
```sqf
_array select -1;
```

### Explanation

`select` can take a negative index to select from the end of the array. This is more efficient than calculating the index from SQF.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _) = target else {
            return Vec::new();
        };
        
        if cmd.to_lowercase() != "select" {
            return Vec::new();
        }

        if let Expression::BinaryCommand(BinaryCommand::Sub, slhs, srhs, _) = &**rhs
            && let Expression::UnaryCommand(UnaryCommand::Named(cmd), array, _) = &**slhs {
                if cmd.to_lowercase() != "count" {
                    return Vec::new();
                }
                if array.source() != lhs.source() {
                    return Vec::new();
                }
                if let Expression::Number(index, _) = &**srhs {
                    if index.0.fract() != 0.0 {
                        return Vec::new();
                    }
                    if index.0 < 1.0 {
                        return Vec::new();
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    #[allow(clippy::cast_sign_loss)]
                    return vec![Arc::new(CodeS27SelectCount::new(
                        index.0 as usize,
                        array.source(),
                        target.full_span(),
                        processed,
                        config.severity(),
                    ))];
                }
            }

        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS27SelectCount {
    index: usize,
    array: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS27SelectCount {
    fn ident(&self) -> &'static str {
        "L-S27"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#select_count")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Using `select` with `count`".to_string()
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn suggestion(&self) -> Option<String> {
        if self.array.contains("select") {
            Some(format!("({}) select -{}", self.array, self.index))
        } else {
            Some(format!("{} select -{}", self.array, self.index))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS27SelectCount {
    #[must_use]
    pub fn new(index: usize, array: String, mut span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        #[allow(clippy::range_plus_one)]
        if array.contains("select") {
            span = span.start - 1..span.end + 1;
        } else {
            span = span.start..span.end + 1;
        }
        Self {
            index,
            array,
            span,
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
