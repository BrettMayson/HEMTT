use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS48IsEqualTypeAll);

impl Lint<LintData> for LintS48IsEqualTypeAll {
    fn ident(&self) -> &'static str {
        "is_equal_type_all"
    }

    fn sort(&self) -> u32 {
        480
    }

    fn description(&self) -> &'static str {
        "Checks for verifying the type of every array element, replaceable with `isEqualTypeAll`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
if ({_x isEqualType 0} count _arr == count _arr) then {};
if (_arr findIf {!(_x isEqualType 0)} == -1) then {};
```
**Correct**
```sqf
if (_arr isEqualTypeAll 0) then {};
```

### Explanation

`isEqualTypeAll` checks the type of every element in one command.

Note that `isEqualTypeAll` returns `false` for an empty array, where the checks above return
`true`, so the replacement is not equivalent for an array that may be empty."
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
        check(target, processed, config)
    }
}

// Pattern 1: {_x isEqualType T} count ARR == count ARR
// Pattern 2: ARR findIf {!(_x isEqualType T)} == -1
// Both are `ARR isEqualTypeAll T`, except on an empty array, see the note on the emitted code.
// The negated forms (`!=`, `> -1`, `>= 0`, ...) map to `!(ARR isEqualTypeAll T)`.

fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(cmd, left, right, _) = target else {
        return codes;
    };

    let Some((array, type_expr, negated)) =
        check_count(cmd, left, right).or_else(|| check_findif(cmd, left, right))
    else {
        return codes;
    };

    codes.push(Arc::new(CodeS48IsEqualTypeAll::new(
        target.full_span(),
        array.source(true),
        type_expr.source(true),
        negated,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// `{_x isEqualType T} count ARR == count ARR`, in either operand order.
fn check_count<'a>(
    cmd: &BinaryCommand,
    left: &'a Expression,
    right: &'a Expression,
) -> Option<(&'a Expression, &'a Expression, bool)> {
    let negated = equality_polarity(cmd)?;
    let (array, type_expr) =
        count_sides(left, right).or_else(|| count_sides(right, left))?;
    Some((array, type_expr, negated))
}

/// Matches `{_x isEqualType T} count ARR` against `count ARR` over the same array.
fn count_sides<'a>(
    counted: &'a Expression,
    total: &'a Expression,
) -> Option<(&'a Expression, &'a Expression)> {
    let Expression::BinaryCommand(BinaryCommand::Named(name), code, array, _) = counted else {
        return None;
    };
    if !name.eq_ignore_ascii_case("count") {
        return None;
    }
    let type_expr = as_x_is_equal_type(unwrap_code_block(code), false)?;

    let Expression::UnaryCommand(UnaryCommand::Named(total_name), total_array, _) = total else {
        return None;
    };
    if !total_name.eq_ignore_ascii_case("count") {
        return None;
    }
    if !expressions_match(array, total_array) {
        return None;
    }
    Some((array, type_expr))
}

/// `ARR findIf {!(_x isEqualType T)}` compared against a `-1` or `0` sentinel.
fn check_findif<'a>(
    cmd: &BinaryCommand,
    left: &'a Expression,
    right: &'a Expression,
) -> Option<(&'a Expression, &'a Expression, bool)> {
    // the comparison is normalised so the findIf is always on the left
    let (find, sentinel, cmd) = match as_findif(left) {
        Some(find) => (find, right, flip(cmd, false)),
        None => (as_findif(right)?, left, flip(cmd, true)),
    };
    let negated = findif_polarity(&cmd, as_number(sentinel)?)?;
    Some((find.0, find.1, negated))
}

/// Matches `ARR findIf {!(_x isEqualType T)}`, returning the array and the type.
fn as_findif(expr: &Expression) -> Option<(&Expression, &Expression)> {
    let Expression::BinaryCommand(BinaryCommand::Named(name), array, code, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("findIf") {
        return None;
    }
    let type_expr = as_x_is_equal_type(unwrap_code_block(code), true)?;
    Some((array, type_expr))
}

/// Swaps a comparison so its operands can be read in the other order.
fn flip(cmd: &BinaryCommand, swap: bool) -> BinaryCommand {
    if !swap {
        return cmd.clone();
    }
    match cmd {
        BinaryCommand::Less => BinaryCommand::Greater,
        BinaryCommand::Greater => BinaryCommand::Less,
        BinaryCommand::LessEq => BinaryCommand::GreaterEq,
        BinaryCommand::GreaterEq => BinaryCommand::LessEq,
        other => other.clone(),
    }
}

/// True when the comparison is an inequality, false when equality, None when neither.
fn equality_polarity(cmd: &BinaryCommand) -> Option<bool> {
    if matches!(cmd, BinaryCommand::Eq)
        || matches!(cmd, BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isEqualTo"))
    {
        return Some(false);
    }
    if matches!(cmd, BinaryCommand::NotEq)
        || matches!(cmd, BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isNotEqualTo"))
    {
        return Some(true);
    }
    None
}

/// `findIf` yields -1 or an index in [0, inf), so each accepted shape below is exactly a test
/// for -1, or its negation. Returns true when the expression means "not all matched".
fn findif_polarity(cmd: &BinaryCommand, sentinel: f32) -> Option<bool> {
    let is = |value: f32| (sentinel - value).abs() < f32::EPSILON;
    if matches!(cmd, BinaryCommand::Eq if is(-1.0))
        || matches!(cmd, BinaryCommand::LessEq if is(-1.0))
        || matches!(cmd, BinaryCommand::Less if is(0.0))
        || matches!(cmd, BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isEqualTo") && is(-1.0))
    {
        return Some(false);
    }
    if matches!(cmd, BinaryCommand::NotEq if is(-1.0))
        || matches!(cmd, BinaryCommand::Greater if is(-1.0))
        || matches!(cmd, BinaryCommand::GreaterEq if is(0.0))
        || matches!(cmd, BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isNotEqualTo") && is(-1.0))
    {
        return Some(true);
    }
    None
}

/// Reads a numeric literal, seeing through a unary minus, as `-1` parses as `Minus` over `1`.
fn as_number(expr: &Expression) -> Option<f32> {
    match expr {
        Expression::Number(value, _) => Some(value.0),
        Expression::UnaryCommand(UnaryCommand::Minus, inner, _) => as_number(inner).map(|v| -v),
        _ => None,
    }
}

/// Matches `_x isEqualType T`, optionally wrapped in a `!`.
fn as_x_is_equal_type(expr: &Expression, negated: bool) -> Option<&Expression> {
    let expr = if negated {
        let Expression::UnaryCommand(UnaryCommand::Not, inner, _) = expr else {
            return None;
        };
        &**inner
    } else {
        expr
    };
    let Expression::BinaryCommand(BinaryCommand::Named(name), lhs, rhs, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("isEqualType") {
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

#[allow(clippy::module_name_repetitions)]
pub struct CodeS48IsEqualTypeAll {
    span: Range<usize>,
    array: String,
    type_expr: String,
    negated: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS48IsEqualTypeAll {
    fn ident(&self) -> &'static str {
        "L-S48"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#is_equal_type_all")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `isEqualTypeAll`")
    }

    fn label_message(&self) -> String {
        String::from("use `isEqualTypeAll`")
    }

    fn note(&self) -> Option<String> {
        // isEqualTypeAll is false for an empty array, so the two disagree there either way round
        Some(String::from(if self.negated {
            "`isEqualTypeAll` returns false for an empty array, so this suggestion returns true where the original returns false"
        } else {
            "`isEqualTypeAll` returns false for an empty array, where this check returns true"
        }))
    }

    fn suggestion(&self) -> Option<String> {
        Some(if self.negated {
            format!("!({} isEqualTypeAll {})", self.array, self.type_expr)
        } else {
            format!("{} isEqualTypeAll {}", self.array, self.type_expr)
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS48IsEqualTypeAll {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        array: String,
        type_expr: String,
        negated: bool,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            array,
            type_expr,
            negated,
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
