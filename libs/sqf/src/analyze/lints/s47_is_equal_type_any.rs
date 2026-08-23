use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, analyze::LintData};

crate::analyze::lint!(LintS47IsEqualTypeAny);

impl Lint<LintData> for LintS47IsEqualTypeAny {
    fn ident(&self) -> &'static str {
        "is_equal_type_any"
    }

    fn sort(&self) -> u32 {
        470
    }

    fn description(&self) -> &'static str {
        "Checks for chained `isEqualType` comparisons, which can be replaced with `isEqualTypeAny`"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
if (_x isEqualType 0 || _x isEqualType "") then {};
```
**Correct**
```sqf
if (_x isEqualTypeAny [0, ""]) then {};
```

### Explanation

`isEqualTypeAny` compares a value against several types in one command, rather than chaining a
separate comparison for each."#
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

        // Walking from the statement, rather than being handed every expression, is what lets
        // this see whether an `||` has another `||` above it. `a || b || c` parses as
        // `(a || b) || c`, so only the outermost node has the whole chain beneath it.
        let mut codes = Vec::new();
        let expression = match target {
            crate::Statement::AssignGlobal(_, expression, _)
            | crate::Statement::AssignLocal(_, expression, _)
            | crate::Statement::Expression(expression, _) => expression,
        };
        walk(expression, false, processed, config, &mut codes);
        codes
    }
}

/// Visits every expression, reporting a chain only at its outermost `||`.
fn walk(
    expression: &Expression,
    parent_is_or: bool,
    processed: &Processed,
    config: &LintConfig,
    codes: &mut Codes,
) {
    if let Expression::BinaryCommand(BinaryCommand::Or, ..) = expression
        && !parent_is_or
    {
        codes.extend(check(expression, processed, config));
    }

    let is_or = matches!(expression, Expression::BinaryCommand(BinaryCommand::Or, ..));

    match expression {
        Expression::BinaryCommand(_, lhs, rhs, _) => {
            walk(lhs, is_or, processed, config, codes);
            walk(rhs, is_or, processed, config, codes);
        }
        Expression::UnaryCommand(_, inner, _) => {
            walk(inner, false, processed, config, codes);
        }
        Expression::Array(items, _) | Expression::ConsumeableArray(items, _) => {
            for item in items {
                walk(item, false, processed, config, codes);
            }
        }
        Expression::Code(statements) => {
            for statement in statements.content() {
                let inner = match statement {
                    crate::Statement::AssignGlobal(_, inner, _)
                    | crate::Statement::AssignLocal(_, inner, _)
                    | crate::Statement::Expression(inner, _) => inner,
                };
                walk(inner, false, processed, config, codes);
            }
        }
        _ => {}
    }
}

// Pattern: x isEqualType a || x isEqualType b || ...
// SQF Structure: (binary ||) of two or more `isEqualType` comparisons sharing a left hand side

fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(BinaryCommand::Or, ..) = target else {
        return codes;
    };
    let mut branches = Vec::new();
    if !flatten_or(target, &mut branches) {
        return codes;
    }

    // every branch must be `x isEqualType <type>` with the same x
    let mut types = Vec::new();
    let mut value: Option<&Expression> = None;
    for branch in branches {
        let Some((lhs, rhs)) = as_is_equal_type(branch) else {
            return codes;
        };
        match value {
            None => value = Some(lhs),
            Some(previous) if expressions_match(previous, lhs) => {}
            Some(_) => return codes,
        }
        types.push(rhs.source(true));
    }

    let Some(value) = value else {
        return codes;
    };
    if types.len() < 2 {
        return codes;
    }

    codes.push(Arc::new(CodeS47IsEqualTypeAny::new(
        target.full_span(),
        value.source(true),
        types,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Flattens a chain of `||` into its branches. Returns false if a branch is itself unusable.
fn flatten_or<'a>(expr: &'a Expression, out: &mut Vec<&'a Expression>) -> bool {
    if let Expression::BinaryCommand(BinaryCommand::Or, left, right, _) = expr {
        return flatten_or(left, out) && flatten_or(right, out);
    }
    out.push(expr);
    true
}

/// Matches `x isEqualType y`, seeing through a short circuit code block on the right of `||`.
fn as_is_equal_type(expr: &Expression) -> Option<(&Expression, &Expression)> {
    let expr = unwrap_code_block(expr);
    let Expression::BinaryCommand(BinaryCommand::Named(name), lhs, rhs, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("isEqualType") {
        return None;
    }
    Some((lhs, rhs))
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS47IsEqualTypeAny {
    span: Range<usize>,
    value: String,
    types: Vec<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS47IsEqualTypeAny {
    fn ident(&self) -> &'static str {
        "L-S47"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#is_equal_type_any")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `isEqualTypeAny`")
    }

    fn label_message(&self) -> String {
        String::from("use `isEqualTypeAny`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{} isEqualTypeAny [{}]",
            self.value,
            self.types.join(", ")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS47IsEqualTypeAny {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        value: String,
        types: Vec<String>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            value,
            types,
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

/// Check if two expressions match, considering variables and numbers
fn expressions_match(expr1: &Expression, expr2: &Expression) -> bool {
    match (expr1, expr2) {
        (Expression::Variable(v1, _), Expression::Variable(v2, _)) => v1 == v2,
        (Expression::Number(n1, _), Expression::Number(n2, _)) => {
            (n1.0 - n2.0).abs() < f32::EPSILON
        }
        _ => false,
    }
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
