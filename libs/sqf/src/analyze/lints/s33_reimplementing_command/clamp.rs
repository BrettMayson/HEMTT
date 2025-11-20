use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern 1: if (_v < min) then {min} else { if (_v > max) then {max} else {_v} }
// Pattern 2: if (_v > max) then {max} else { if (_v < min) then {min} else {_v} }
// Pattern 3: if (_v < min) then {min} else { _v max _max }
// Pattern 4: if (_v > max) then {max} else { _v min _min }
// Should become: _min max _v min _max
// This is a clamping pattern that restricts a value between min and max

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for "then" binary command
    let Expression::BinaryCommand(BinaryCommand::Named(cmd), if_expr, code, _) = target else {
        return codes;
    };

    if !cmd.eq_ignore_ascii_case("then") {
        return codes;
    }

    // Check for "if" unary command
    let Expression::UnaryCommand(UnaryCommand::Named(if_cmd), outer_condition, _) = &**if_expr else {
        return codes;
    };

    if !if_cmd.eq_ignore_ascii_case("if") {
        return codes;
    }

    // Check for else branch
    let Expression::BinaryCommand(BinaryCommand::Else, outer_then, outer_else, _) = &**code else {
        return codes;
    };

    // Get the outer comparison
    let Expression::BinaryCommand(outer_comparison, outer_left, outer_right, _) = &**outer_condition else {
        return codes;
    };

    // Try to match the pattern
    if let Some((min, value, max)) = check_clamp_pattern(
        outer_comparison,
        outer_left,
        outer_right,
        outer_then,
        outer_else,
    ) {
        let min_text = min.source(true);
        let value_text = value.source(true);
        let max_text = max.source(true);

        codes.push(Arc::new(CodeS33ReimplementingCommandClamp::new(
            target.full_span(),
            min_text,
            value_text,
            max_text,
            processed,
            config.severity(),
        )) as Arc<dyn Code>);
    }

    codes
}

// Helper function to check various clamp patterns
fn check_clamp_pattern<'a>(
    outer_comparison: &BinaryCommand,
    outer_left: &'a Expression,
    outer_right: &'a Expression,
    outer_then: &'a Expression,
    outer_else: &'a Expression,
) -> Option<(&'a Expression, &'a Expression, &'a Expression)> {
    match outer_comparison {
        BinaryCommand::Less | BinaryCommand::LessEq => {
            // Pattern: if (_v < min) then {min} else { ... }
            let value = outer_left;
            let min = outer_right;

            // Outer then branch should be: {min}
            if !super::expressions_match(min, outer_then, true) {
                return None;
            }

            // Check else branch for nested if-then-else or max command
            let inner_expr = super::unwrap_code_block(outer_else);

            // Try nested if-then-else pattern: if (_v > max) then {max} else {_v}
            if let Expression::BinaryCommand(BinaryCommand::Named(then_cmd), inner_if_expr, inner_code, _) = inner_expr
                && then_cmd.eq_ignore_ascii_case("then")
                && let Expression::UnaryCommand(UnaryCommand::Named(inner_if_cmd), inner_condition, _) = &**inner_if_expr
                && inner_if_cmd.eq_ignore_ascii_case("if")
                && let Expression::BinaryCommand(BinaryCommand::Else, inner_then, inner_else, _) = &**inner_code
                && let Expression::BinaryCommand(inner_comparison, inner_left, inner_right, _) = &**inner_condition
                && matches!(inner_comparison, BinaryCommand::Greater | BinaryCommand::GreaterEq)
                && super::expressions_match(value, inner_left, false)
                && super::expressions_match(inner_right, inner_then, true)
                && super::expressions_match(value, inner_else, true)
            {
                return Some((min, value, inner_right));
            }

            // Try max command pattern: _v max _max
            if let Expression::BinaryCommand(BinaryCommand::Max, max_left, max_right, _) = inner_expr
                && super::expressions_match(value, max_left, true)
            {
                return Some((min, value, max_right));
            }

            None
        }
        BinaryCommand::Greater | BinaryCommand::GreaterEq => {
            // Pattern: if (_v > max) then {max} else { ... }
            let value = outer_left;
            let max = outer_right;

            // Outer then branch should be: {max}
            if !super::expressions_match(max, outer_then, true) {
                return None;
            }

            // Check else branch for nested if-then-else or min command
            let inner_expr = super::unwrap_code_block(outer_else);

            // Try nested if-then-else pattern: if (_v < min) then {min} else {_v}
            if let Expression::BinaryCommand(BinaryCommand::Named(then_cmd), inner_if_expr, inner_code, _) = inner_expr
                && then_cmd.eq_ignore_ascii_case("then")
                && let Expression::UnaryCommand(UnaryCommand::Named(inner_if_cmd), inner_condition, _) = &**inner_if_expr
                && inner_if_cmd.eq_ignore_ascii_case("if")
                && let Expression::BinaryCommand(BinaryCommand::Else, inner_then, inner_else, _) = &**inner_code
                && let Expression::BinaryCommand(inner_comparison, inner_left, inner_right, _) = &**inner_condition
                && matches!(inner_comparison, BinaryCommand::Less | BinaryCommand::LessEq)
                && super::expressions_match(value, inner_left, false)
                && super::expressions_match(inner_right, inner_then, true)
                && super::expressions_match(value, inner_else, true)
            {
                return Some((inner_right, value, max));
            }

            // Try min command pattern: _v min _min
            if let Expression::BinaryCommand(BinaryCommand::Min, min_left, min_right, _) = inner_expr
                && super::expressions_match(value, min_left, true)
            {
                return Some((min_right, value, max));
            }

            None
        }
        _ => None,
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandClamp {
    span: Range<usize>,
    min: String,
    value: String,
    max: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandClamp {
    fn ident(&self) -> &'static str {
        "L-S33-CLAMP"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with clamping pattern")
    }

    fn label_message(&self) -> String {
        String::from("use clamp pattern")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} max {} min {}", self.min, self.value, self.max))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandClamp {
    #[must_use]
    pub fn new(span: Range<usize>, min: String, value: String, max: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            min,
            value,
            max,
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
