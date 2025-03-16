use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, Statement};

crate::analyze::lint!(LintS26ShortCircuitBoolVar);

impl Lint<LintData> for LintS26ShortCircuitBoolVar {
    fn ident(&self) -> &'static str {
        "short_circuit_bool_var"
    }

    fn sort(&self) -> u32 {
        260
    }

    fn description(&self) -> &'static str {
        "Checks for inefficent short ciruit evaulation"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
if (_test1 && {_test2}) then { };
```
**Correct**
```sqf
if (_test1 && _test2) then { };
```

### Explanation

Short circuit evaultion on a variable that is a boolean is inefficient
False positives are possible if the var could be undefined, e.g.:
```sqf
(!isNil "z") && {z}
```
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(hemtt_common::config::LintEnabled::Pedantic)
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
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(cmd, _left, right, _) = target else {
            return Vec::new();
        };
        if !(matches!(cmd, BinaryCommand::Or) || matches!(cmd, BinaryCommand::And)) {
            return Vec::new();
        }
        let Expression::Code(statements)= &**right else {
            return Vec::new();
        };
        if statements.content().len() != 1 { 
            return Vec::new()
        }
        let Statement::Expression(Expression::Variable(ref _var_name, _), ref range) = statements.content()[0] else {
            return Vec::new();
        };
        vec![Arc::new(CodeS26ShortCircuitBoolVar::new(
            range.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS26ShortCircuitBoolVar {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS26ShortCircuitBoolVar {
    fn ident(&self) -> &'static str {
        "L-S26"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#short_circuit_bool_var")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Inefficent short circuit evaulation".to_string()
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn note(&self) -> Option<String> {
        Some("remove the { } and use the variable directly (if safe to do so)".to_string())
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}
impl CodeS26ShortCircuitBoolVar {
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
