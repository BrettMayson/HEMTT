use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, Statement, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS57ParamsInLoop);

impl Lint<LintData> for LintS57ParamsInLoop {
    fn ident(&self) -> &'static str {
        "params_in_loop"
    }
    fn sort(&self) -> u32 {
        570
    }
    fn description(&self) -> &'static str {
        "Checks for `params` that don't use `_x` inside `forEach`, `apply` or `select` loops"
    }
    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
{
    params ["_value"];
} forEach _values;
```

**Correct**
```sqf
{
    _x params ["_value"];
} forEach _values;
```
"#
    }
    fn default_config(&self) -> LintConfig {
        LintConfig::help() // possible legit use but very unlikely and lint can be disabled
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
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) = target else {
            return Vec::new();
        };

        let body = if command.eq_ignore_ascii_case("forEach") {
            lhs
        } else if command.eq_ignore_ascii_case("apply") || command.eq_ignore_ascii_case("select") {
            rhs
        } else {
            return Vec::new();
        };
        let Expression::Code(body) = body.as_ref() else {
            return Vec::new();
        };

        let mut codes: Codes = Vec::new();
        // Only consider top-level statements, not nested expressions which may be conditional or independent code blocks
        for statement in body.content() { 
            let Statement::Expression(Expression::UnaryCommand(UnaryCommand::Named(name), _, span), _) = statement else {
                continue;
            };
            if name.eq_ignore_ascii_case("params") {
                codes.push(Arc::new(CodeS57ParamsInLoop::new(
                    span.clone(),
                    processed,
                    config.severity(),
                )));
            }
        }
        codes
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS57ParamsInLoop {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS57ParamsInLoop {
    fn ident(&self) -> &'static str {
        "L-S57"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#params_in_loop")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Use binary `params` on `_x` inside `forEach` or `apply` loops".to_string()
    }

    fn label_message(&self) -> String {
        "unary `params` reads `_this`, not the current loop element".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS57ParamsInLoop {
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