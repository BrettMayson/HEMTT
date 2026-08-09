use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, Statement, UnaryCommand};

// Pattern 1: if !(x in arr) then {arr pushBack x}        -> arr pushBackUnique x
// Pattern 2: if !(x in arr) then {arr pushBackUnique x}  -> the check is already done by the command
//
// `in` and `pushBackUnique` are both case sensitive on strings, so the two are equivalent.

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
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
    if !super::expressions_match(haystack, array, false)
        || !super::expressions_match(needle, value, false)
    {
        return codes;
    }

    codes.push(Arc::new(CodeS33ReimplementingCommandPushBackUnique::new(
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
pub struct CodeS33ReimplementingCommandPushBackUnique {
    span: Range<usize>,
    array: String,
    value: String,
    already_unique: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandPushBackUnique {
    fn ident(&self) -> &'static str {
        "L-S33-PUSHBACKUNIQUE"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
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

impl CodeS33ReimplementingCommandPushBackUnique {
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
