use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::SqfLintData, BinaryCommand, Expression};

crate::lint!(LintS22InVehicleCheck);

impl Lint<SqfLintData> for LintS22InVehicleCheck {
    fn ident(&self) -> &str {
        "this_call"
    }

    fn sort(&self) -> u32 {
        220
    }

    fn description(&self) -> &str {
        "Checks for usage of `_this call`, where `_this` is not necessary"
    }

    fn documentation(&self) -> &str {
        r"### Example

**Incorrect**
```sqf
_this call _my_function;
```
**Correct**
```sqf
call _my_function;
```

### Explanation

When using `call`, the called code will inherit `_this` from the calling code. This means that `_this` is not necessary in the call, and can be omitted for better performance.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(false)
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
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, _, _) = target else {
            return Vec::new();
        };
        
        if cmd != "call" {
            return Vec::new();
        }

        if matches!(&**lhs, Expression::Variable(name, _) if name == "_this") {
            return vec![Arc::new(CodeS22ThisCall::new(
                lhs.span(),
                processed,
                config.severity(),
            ))];
        }

        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS22ThisCall {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS22ThisCall {
    fn ident(&self) -> &'static str {
        "L-S22"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#this_call")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Unnecessary `_this` in `call`".to_string()
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn note(&self) -> Option<String> {
        Some("`call` inherits `_this` from the calling code".to_string())
    }

    fn help(&self) -> Option<String> {
        Some("Remove `_this` from the call".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS22ThisCall {
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
