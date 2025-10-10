use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, Statement, UnaryCommand};

crate::analyze::lint!(LintS31IsNilGetVariable);

impl Lint<LintData> for LintS31IsNilGetVariable {
    fn ident(&self) -> &'static str {
        "isnil_getvariable"
    }

    fn sort(&self) -> u32 {
        260
    }

    fn description(&self) -> &'static str {
        "Checks for inefficent isNil/getVariable pattern"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
isNil {x getVariable 'varName'};
```
**Correct**
```sqf
x isNil 'varName';
```

### Explanation

Using `isNil`'s alterante syntax is slightly faster
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help() // .with_enabled(hemtt_common::config::LintEnabled::Pedantic)
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
        let Expression::UnaryCommand(UnaryCommand::Named(cmd), right, _) = target else {
            return Vec::new();
        };
        if !cmd.eq_ignore_ascii_case("isnil") {
            return Vec::new();
        }
        let Expression::Code(statements) = &**right else {
            return Vec::new();
        };
        let [Statement::Expression(Expression::BinaryCommand(BinaryCommand::Named(getvar_cmd), lhs, rhs, getvar_span), _)] = statements.content() else {
            return Vec::new();
        };
        if !getvar_cmd.eq_ignore_ascii_case("getvariable") {
            return Vec::new();
        }

        vec![Arc::new(CodeS31IsNilGetVariable::new(
            lhs.source(),
            rhs.source(),
            getvar_span.clone(),
            processed,
            config.severity(),
        ))]
    }
}

pub struct CodeS31IsNilGetVariable {
    lhs: String,
    rhs: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS31IsNilGetVariable {
    fn ident(&self) -> &'static str {
        "L-S31"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#isnil_getvariable")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Use isNil directly".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} isNil {}", self.lhs, self.rhs))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}
impl CodeS31IsNilGetVariable {
    #[must_use]
    pub fn new(lhs: String, rhs: String, span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            lhs,
            rhs,
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
