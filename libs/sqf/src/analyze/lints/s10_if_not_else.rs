use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::SqfLintData, BinaryCommand, Expression, UnaryCommand};

crate::lint!(LintS10IfNotElse);

impl Lint<SqfLintData> for LintS10IfNotElse {
    fn ident(&self) -> &str {
        "if_not_else"
    }

    fn sort(&self) -> u32 {
        80
    }

    fn description(&self) -> &str {
        "Checks for unneeded not"
    }

    fn documentation(&self) -> &str {
r"### Example

**Incorrect**
```sqf
if (!alive player) then { player } else { objNull };
```
**Correct**
```sqf
if (alive player) then { objNull } else { player };
```

### Explanation

"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::pedantic()
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

        let Some(processed) = processed else {
            return Vec::new();
        };
        if let Expression::BinaryCommand(BinaryCommand::Named(name), if_cmd, code, _) = target {
            if name.to_lowercase() == "then" {
                let Expression::UnaryCommand(UnaryCommand::Named(_), condition, _) = &**if_cmd else {
                    return Vec::new();
                };
                if let Expression::BinaryCommand(BinaryCommand::Else, _, _, _) = &**code {
                    if let Expression::UnaryCommand(UnaryCommand::Not, _, _) = &**condition
                    {
                    return vec![Arc::new(CodeS10IfNot::new(
                            if_cmd.span(),
                            "if ! else - consider removing not and swapping else".to_string(),
                            processed,
                            config.severity(),
                        ))];
                    }
                }
            }
        }
        Vec::new()
    }
}


#[allow(clippy::module_name_repetitions)]
pub struct CodeS10IfNot {
    span: Range<usize>,
    problem: String, //todo

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS10IfNot {
    fn ident(&self) -> &'static str {
        "L-S10"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#if_not_else")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        self.problem.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS10IfNot {
    #[must_use]
    pub fn new(span: Range<usize>, problem: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            problem,

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
