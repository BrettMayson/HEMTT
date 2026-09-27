use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, LintEnabled};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{
    BinaryCommand, Expression, Statement, Statements, analyze::LintData,
};

crate::analyze::lint!(LintS41ForeachApply);

impl Lint<LintData> for LintS41ForeachApply {
    fn ident(&self) -> &'static str {
        "foreach_apply"
    }

    fn sort(&self) -> u32 {
        410
    }

    fn description(&self) -> &'static str {
        "Checks for forEach loops where `_forEachIndex` is never used"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
{
    _values pushBack _x;
} forEach bigArray;
```

**Correct**
```sqf
bigArray apply {
    _values pushBack _x;
};
```

### Explanation

When a `forEach` loop never uses `_forEachIndex`, it can often be replaced with `apply`, which is the more idiomatic and typically faster form for array iteration.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(LintEnabled::Disabled)
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Statement;

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
        let Statement::Expression(expression, _) = target else { // skip assignements
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Named(command), lhs, _, _) = expression else {
            return Vec::new();
        };
        if !command.eq_ignore_ascii_case("foreach") {
            return Vec::new();
        }
        let Expression::Code(body) = lhs.as_ref() else {
            return Vec::new();
        };
        if body_uses_for_each_index(body) {
            return Vec::new();
        }
        vec![Arc::new(CodeS41ForeachApply::new(
            expression.span(),
            processed,
            config.severity(),
        ))]
    }
}

fn body_uses_for_each_index(statements: &Statements) -> bool {
    statements.walk_expressions().iter().any(|expr| {
        matches!(expr, Expression::Variable(name, _) if name.eq_ignore_ascii_case("_forEachIndex"))
    })
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS41ForeachApply {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS41ForeachApply {
    fn ident(&self) -> &'static str {
        "L-S41"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#foreach_apply")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Use `apply` instead of `forEach` when `_forEachIndex` is unused".to_string()
    }

    fn label_message(&self) -> String {
        "forEach loop can use apply".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("replace with `array apply { ... }`".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS41ForeachApply {
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
