use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern 1: {_x isKindOf TYPE} count ARR  ->  TYPE countType ARR
// Pattern 2: {side _x == SIDE} count ARR   ->  SIDE countSide ARR
//
// Pattern 2 only matches `side _x`. `side group _x` is a different value, a unit's own side
// can differ from its group's side, so it is deliberately left alone.

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(BinaryCommand::Named(cmd), code, array, _) = target else {
        return codes;
    };
    if !cmd.eq_ignore_ascii_case("count") {
        return codes;
    }
    let inner = super::unwrap_code_block(code);

    let found = as_kind_of(inner)
        .map(|type_expr| ("countType", type_expr))
        .or_else(|| as_side_comparison(inner).map(|side_expr| ("countSide", side_expr)));

    let Some((command, operand)) = found else {
        return codes;
    };

    codes.push(Arc::new(CodeS33ReimplementingCommandCountTypeSide::new(
        target.full_span(),
        command,
        operand.source(true),
        array.source(true),
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Matches `_x isKindOf TYPE`, returning TYPE.
fn as_kind_of(expr: &Expression) -> Option<&Expression> {
    let Expression::BinaryCommand(BinaryCommand::Named(name), lhs, rhs, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("isKindOf") {
        return None;
    }
    if !is_x(lhs) {
        return None;
    }
    Some(rhs)
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
    name.eq_ignore_ascii_case("side") && is_x(inner)
}

/// The magic variable is spelled exactly `_x`, anything else is a different variable.
fn is_x(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(name, _) if name == "_x")
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandCountTypeSide {
    span: Range<usize>,
    command: &'static str,
    operand: String,
    array: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandCountTypeSide {
    fn ident(&self) -> &'static str {
        "L-S33-COUNTTYPESIDE"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("code can be replaced with `{}`", self.command)
    }

    fn label_message(&self) -> String {
        format!("use `{}`", self.command)
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{} {} {}",
            self.operand, self.command, self.array
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandCountTypeSide {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        command: &'static str,
        operand: String,
        array: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            command,
            operand,
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
