use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern 1: if (x >= 0) then {x} else {-x}
// Pattern 2: if (x < 0) then {-x} else {x}
// SQF Structure: (if condition) then {code} else {code}
// Parser: BinaryCommand("then", UnaryCommand("if", condition), BinaryCommand(Else, then_code, else_code))

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

    // Check if condition is a comparison (>= 0 or < 0)
    let Expression::BinaryCommand(comparison_op, left, right, _) = &**condition else {
        return codes;
    };

    // Check if comparing to 0
    let Expression::Number(num, _) = &**right else {
        return codes;
    };

    if num.0.abs() > f32::EPSILON {
        return codes;
    }

    // Get the variable being compared
    let var_expr = &**left;

    // Check if then and else branches match the abs pattern
    match comparison_op {
        BinaryCommand::GreaterEq | BinaryCommand::Greater => {
            // if (x >= 0) then {x} else {-x}
            if super::expressions_match(var_expr, then_branch, true) && is_negation_of(var_expr, else_branch) {
                let var_text = var_expr.source(true);
                codes.push(Arc::new(CodeS33ReimplementingCommandAbs::new(
                    target.full_span(),
                    var_text,
                    processed,
                    config.severity(),
                )));
            }
        }
        BinaryCommand::Less | BinaryCommand::LessEq => {
            // if (x < 0) then {-x} else {x}
            if is_negation_of(var_expr, then_branch) && super::expressions_match(var_expr, else_branch, true) {
                let var_text = var_expr.source(true);
                codes.push(Arc::new(CodeS33ReimplementingCommandAbs::new(
                    target.full_span(),
                    var_text,
                    processed,
                    config.severity(),
                )));
            }
        }
        _ => {}
    }

    codes
}

fn is_negation_of(expr1: &Expression, expr2: &Expression) -> bool {
    let expr2 = super::unwrap_code_block(expr2);
    
    if let Expression::UnaryCommand(UnaryCommand::Minus, inner, _) = expr2 {
        super::expressions_match(expr1, inner, true)
    } else {
        false
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandAbs {
    span: Range<usize>,
    var: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandAbs {
    fn ident(&self) -> &'static str {
        "L-S33-ABS"
    }
    
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `abs`")
    }

    fn label_message(&self) -> String {
        String::from("use `abs`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("abs {}", self.var))
    }
}

impl CodeS33ReimplementingCommandAbs {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        var: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            var,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic =
            Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
