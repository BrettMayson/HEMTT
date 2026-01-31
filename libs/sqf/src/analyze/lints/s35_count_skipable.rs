use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, LintEnabled};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression};

crate::analyze::lint!(LintS35CountSkippable);

impl Lint<LintData> for LintS35CountSkippable {
    fn ident(&self) -> &'static str {
        "count_skipable"
    }
    fn sort(&self) -> u32 {
        350
    }
    fn description(&self) -> &'static str {
        "Checks for use of `count` when `findIf` could possibly be used instead"
    }
    fn documentation(&self) -> &'static str {
        r"
        ### Example

**Incorrect**
```sqf
{alive _x} count allUnits > 0 // has to check every single unit even if the first is alive
```
**Correct**
```sqf
allUnits findIf {alive _x} != -1
```
### Explanation

When checking if any elements in an array match a condition, using `findIf` can be more efficient than `count`, as `findIf` can stop searching once a match is found.

Check [the wiki](https://community.bistudio.com/wiki/findIf) to learn more about `findIf` optimization.
"
    }

    fn default_config(&self) -> LintConfig {
        // Disabled by default because `count` could be appropriate in some cases
        // e.g. { x setVar y; alive x} count allUnits == 0;
        LintConfig::help().with_enabled(LintEnabled::Pedantic)
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
        /// Check if the first expression is B:count and the second is num0
        fn check_count_0(first: &Expression, second: &Expression) -> bool {
            let Expression::BinaryCommand(BinaryCommand::Named(cname), _, _, _) = first else {
                return false;
            };
            let Expression::Number(num, _) = second else {
                return false;
            };
           cname.as_str() == "count" && num.0 == 0.0
        }
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
        let Expression::BinaryCommand(bcmd, lhs, rhs, span) = target else {
            return Vec::new();
        };
        if !COMPARISON_CMDS.contains(&bcmd.as_str()) {
            return Vec::new();
        }
        if !(check_count_0(lhs, rhs) || check_count_0(rhs, lhs)) {
            return Vec::new();
        }
        vec![Arc::new(CodeS35CountSkippable::new(
            span.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS35CountSkippable {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS35CountSkippable {
    fn ident(&self) -> &'static str {
        "L-S35"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#count_skipable")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        "count could be replaced with findIf".to_string()
    }
    fn label_message(&self) -> String {
        "count compared to 0".to_string()
    }
    fn note(&self) -> Option<String> {
        Some("using `findIf` can skip unnecessary checks once a match is found".to_string())
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS35CountSkippable {
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
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
