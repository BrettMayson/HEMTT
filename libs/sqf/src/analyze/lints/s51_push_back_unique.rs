use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, Statement, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS51PushBackUnique);

impl Lint<LintData> for LintS51PushBackUnique {
    fn ident(&self) -> &'static str {
        "push_back_unique"
    }

    fn sort(&self) -> u32 {
        510
    }

    fn description(&self) -> &'static str {
        "Checks for guarding `pushBack` with `in`, which can be replaced with `pushBackUnique`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
if !(_value in _list) then {_list pushBack _value};
```
**Correct**
```sqf
_list pushBackUnique _value;
```

### Explanation

`pushBackUnique` only adds the element if it is not already in the array, so the surrounding
`in` check is doing work the command already does.

The same shape already using `pushBackUnique` is reported as a redundant check. `in` and
`pushBackUnique` are both case sensitive on strings, so the replacement is exact."
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

// Pattern 1: if !(x in arr) then {arr pushBack x}        -> arr pushBackUnique x
// Pattern 2: if !(x in arr) then {arr pushBackUnique x}  -> the check is already done by the command
//
// `in` and `pushBackUnique` are both case sensitive on strings, so the two are equivalent.

fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // if ... then {...}, with no else branch
    let Expression::BinaryCommand(BinaryCommand::Named(then_cmd), if_expr, code, _) = target else {
        return codes;
    };
    if !then_cmd.eq_ignore_ascii_case("then") {
        return codes;
    }
    let Expression::UnaryCommand(UnaryCommand::Named(if_cmd), condition, _) = &**if_expr else {
        return codes;
    };
    if !if_cmd.eq_ignore_ascii_case("if") {
        return codes;
    }

    let Some((needle, haystack)) = as_negated_in(condition) else {
        return codes;
    };
    let Some((array, value, already_unique)) = as_push(code) else {
        return codes;
    };

    // the array searched has to be the array pushed to, and the value likewise
    if !expressions_match(haystack, array)
        || !expressions_match(needle, value)
    {
        return codes;
    }

    codes.push(Arc::new(CodeS51PushBackUnique::new(
        target.full_span(),
        array.source(true),
        value.source(true),
        already_unique,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Matches `!(x in arr)`, returning the needle and the haystack.
fn as_negated_in(expr: &Expression) -> Option<(&Expression, &Expression)> {
    let Expression::UnaryCommand(UnaryCommand::Not, inner, _) = expr else {
        return None;
    };
    let Expression::BinaryCommand(BinaryCommand::Named(name), needle, haystack, _) = &**inner
    else {
        return None;
    };
    if !name.eq_ignore_ascii_case("in") {
        return None;
    }
    Some((needle, haystack))
}

/// Matches a lone `arr pushBack x` or `arr pushBackUnique x` in the then branch.
/// The bool is true when it is already `pushBackUnique`, so the guard is simply redundant.
fn as_push(expr: &Expression) -> Option<(&Expression, &Expression, bool)> {
    let Expression::Code(statements) = expr else {
        return None;
    };
    // anything else in the branch means the guard is doing more than the command would
    if statements.content().len() != 1 {
        return None;
    }
    let Statement::Expression(Expression::BinaryCommand(BinaryCommand::Named(name), array, value, _), _) =
        &statements.content()[0]
    else {
        return None;
    };
    if name.eq_ignore_ascii_case("pushBack") {
        return Some((array, value, false));
    }
    if name.eq_ignore_ascii_case("pushBackUnique") {
        return Some((array, value, true));
    }
    None
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS51PushBackUnique {
    span: Range<usize>,
    array: String,
    value: String,
    already_unique: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS51PushBackUnique {
    fn ident(&self) -> &'static str {
        "L-S51"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#push_back_unique")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        if self.already_unique {
            String::from("`pushBackUnique` already checks this")
        } else {
            String::from("code can be replaced with `pushBackUnique`")
        }
    }

    fn label_message(&self) -> String {
        if self.already_unique {
            String::from("redundant check")
        } else {
            String::from("use `pushBackUnique`")
        }
    }

    fn note(&self) -> Option<String> {
        if self.already_unique {
            return Some(String::from(
                "`pushBackUnique` only adds the element if it is not already in the array",
            ));
        }
        None
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} pushBackUnique {}", self.array, self.value))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS51PushBackUnique {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        array: String,
        value: String,
        already_unique: bool,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            array,
            value,
            already_unique,
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
