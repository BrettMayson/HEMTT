use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, analyze::LintData};

crate::analyze::lint!(LintS49CountType);

impl Lint<LintData> for LintS49CountType {
    fn ident(&self) -> &'static str {
        "count_type"
    }

    fn sort(&self) -> u32 {
        490
    }

    fn description(&self) -> &'static str {
        "Checks for counting objects of a type, which can be replaced with `countType`"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
private _tanks = {_x isKindOf "Tank"} count _units;
```
**Correct**
```sqf
private _tanks = "Tank" countType _units;
```

### Explanation

`countType` counts how many objects in an array are of a given type, including parent classes,
in a single command."#
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
    type Target = Expression;

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

        // {_x isKindOf TYPE} count ARR
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), code, array, _) = target else {
            return Vec::new();
        };
        if !cmd.eq_ignore_ascii_case("count") {
            return Vec::new();
        }
        let Some(type_expr) = as_kind_of(unwrap_code_block(code)) else {
            return Vec::new();
        };

        vec![Arc::new(CodeS49CountType::new(
            target.full_span(),
            type_expr.source(true),
            array.source(true),
            processed,
            config.severity(),
        ))]
    }
}

/// Matches `_x isKindOf TYPE`, returning TYPE.
fn as_kind_of(expr: &Expression) -> Option<&Expression> {
    let Expression::BinaryCommand(BinaryCommand::Named(name), lhs, rhs, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("isKindOf") {
        return None;
    }
    // the magic variable is spelled exactly `_x`, anything else is a different variable
    let Expression::Variable(var, _) = &**lhs else {
        return None;
    };
    if var != "_x" {
        return None;
    }
    Some(rhs)
}

/// Unwrap a code block to get the inner expression
fn unwrap_code_block(expr: &Expression) -> &Expression {
    if let Expression::Code(statements) = expr
        && let Some(crate::Statement::Expression(inner, _)) = statements.content().first()
    {
        return inner;
    }
    expr
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS49CountType {
    span: Range<usize>,
    type_name: String,
    array: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS49CountType {
    fn ident(&self) -> &'static str {
        "L-S49"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#count_type")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `countType`")
    }

    fn label_message(&self) -> String {
        String::from("use `countType`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} countType {}", self.type_name, self.array))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS49CountType {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        type_name: String,
        array: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            type_name,
            array,
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
