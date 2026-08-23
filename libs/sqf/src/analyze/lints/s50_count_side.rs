use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS50CountSide);

impl Lint<LintData> for LintS50CountSide {
    fn ident(&self) -> &'static str {
        "count_side"
    }

    fn sort(&self) -> u32 {
        500
    }

    fn description(&self) -> &'static str {
        "Checks for counting units of a side, which can be replaced with `countSide`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
private _west = {side _x == west} count _units;
```
**Correct**
```sqf
private _west = west countSide _units;
```

### Explanation

`countSide` counts how many units in an array belong to a given side in a single command.

Only `side _x` is matched. A unit's own side can differ from its group's, so `side group _x`
and `side leader _x` are left alone."
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

        // {side _x == SIDE} count ARR
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), code, array, _) = target else {
            return Vec::new();
        };
        if !cmd.eq_ignore_ascii_case("count") {
            return Vec::new();
        }
        let Some(type_expr) = as_side_comparison(unwrap_code_block(code)) else {
            return Vec::new();
        };

        vec![Arc::new(CodeS50CountSide::new(
            target.full_span(),
            type_expr.source(true),
            array.source(true),
            processed,
            config.severity(),
        ))]
    }
}

/// Matches `side _x == SIDE` in either operand order, returning SIDE.
fn as_side_comparison(expr: &Expression) -> Option<&Expression> {
    let Expression::BinaryCommand(cmd, lhs, rhs, _) = expr else {
        return None;
    };
    let is_eq = matches!(cmd, BinaryCommand::Eq)
        || matches!(cmd, BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isEqualTo"));
    if !is_eq {
        return None;
    }
    if is_side_of_x(lhs) {
        return Some(rhs);
    }
    if is_side_of_x(rhs) {
        return Some(lhs);
    }
    None
}

/// Matches `side _x`, and nothing else. `side group _x` is a different value entirely.
fn is_side_of_x(expr: &Expression) -> bool {
    let Expression::UnaryCommand(UnaryCommand::Named(name), inner, _) = expr else {
        return false;
    };
    // the magic variable is spelled exactly `_x`, anything else is a different variable
    name.eq_ignore_ascii_case("side")
        && matches!(&**inner, Expression::Variable(var, _) if var == "_x")
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
pub struct CodeS50CountSide {
    span: Range<usize>,
    side: String,
    array: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS50CountSide {
    fn ident(&self) -> &'static str {
        "L-S50"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#count_side")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `countSide`")
    }

    fn label_message(&self) -> String {
        String::from("use `countSide`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} countSide {}", self.side, self.array))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS50CountSide {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        side: String,
        array: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            side,
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
