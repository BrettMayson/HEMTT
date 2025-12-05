use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern 1: if (x > y) then {x} else {y}
// Pattern 2: if (x >= y) then {x} else {y}
// Pattern 3: [x, y] select (x < y)
// SQF Structure: (if condition) then {code} else {code}

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for select pattern: [x, y] select (x < y)
    if let Expression::BinaryCommand(BinaryCommand::Select, array, condition, _) = target
        && let Some((left_expr, right_expr)) = check_select_pattern(array, condition)
    {
        let left_text = left_expr.source(true);
        let right_text = right_expr.source(true);
        codes.push(Arc::new(CodeS33ReimplementingCommandMax::new(
            target.full_span(),
            left_text,
            right_text,
            processed,
            config.severity(),
        )) as Arc<dyn Code>);
        return codes;
    }

    // Check for "then" binary command
    let Expression::BinaryCommand(BinaryCommand::Named(cmd), if_expr, code, _) = target else {
        return codes;
    };

    if !cmd.eq_ignore_ascii_case("then") {
        return codes;
    }

    // Check for "if" unary command
    let Expression::UnaryCommand(UnaryCommand::Named(if_cmd), condition, _) = &**if_expr else {
        return codes;
    };

    if !if_cmd.eq_ignore_ascii_case("if") {
        return codes;
    }

    // Check for else branch
    let Expression::BinaryCommand(BinaryCommand::Else, then_branch, else_branch, _) = &**code else {
        return codes;
    };

    // Check if condition is a comparison (> or >=)
    let Expression::BinaryCommand(comparison_op, left, right, _) = &**condition else {
        return codes;
    };

    // Check if then and else branches match the max pattern
    match comparison_op {
        BinaryCommand::Greater | BinaryCommand::GreaterEq => {
            // if (x > y) then {x} else {y}
            if super::expressions_match(left, then_branch, true)
                && super::expressions_match(right, else_branch, true)
            {
                let left_text = left.source(true);
                let right_text = right.source(true);
                codes.push(Arc::new(CodeS33ReimplementingCommandMax::new(
                    target.full_span(),
                    left_text,
                    right_text,
                    processed,
                    config.severity(),
                )) as Arc<dyn Code>);
            }
        }
        BinaryCommand::Less | BinaryCommand::LessEq => {
            // if (x < y) then {y} else {x}
            if super::expressions_match(right, then_branch, true)
                && super::expressions_match(left, else_branch, true)
            {
                let left_text = left.source(true);
                let right_text = right.source(true);
                codes.push(Arc::new(CodeS33ReimplementingCommandMax::new(
                    target.full_span(),
                    left_text,
                    right_text,
                    processed,
                    config.severity(),
                )) as Arc<dyn Code>);
            }
        }
        _ => {}
    }

    codes
}

// Check for select pattern: [x, y] select (x < y) -> x max y
// Returns Some((left_expr, right_expr)) if pattern matches
fn check_select_pattern<'a>(array: &'a Expression, condition: &'a Expression) -> Option<(&'a Expression, &'a Expression)> {
    // Check if array is [x, y]
    let Expression::Array(elements, _) = array else {
        return None;
    };

    if elements.len() != 2 {
        return None;
    }

    let first = &elements[0];
    let second = &elements[1];

    // Check if condition is a comparison (< or <=)
    let Expression::BinaryCommand(comparison_op, left, right, _) = condition else {
        return None;
    };

    // [x, y] select (x < y) means: if x < y, return index 1 (y), else return index 0 (x)
    // This is equivalent to: max(x, y)
    if matches!(comparison_op, BinaryCommand::Less | BinaryCommand::LessEq)
        && super::expressions_match(left, first, false)
        && super::expressions_match(right, second, false)
    {
        Some((first, second))
    } else {
        None
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandMax {
    span: Range<usize>,
    left: String,
    right: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandMax {
    fn ident(&self) -> &'static str {
        "L-S33-MAX"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `max`")
    }

    fn label_message(&self) -> String {
        String::from("use `max`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} max {}", self.left, self.right))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandMax {
    #[must_use]
    pub fn new(span: Range<usize>, left: String, right: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            left,
            right,
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
