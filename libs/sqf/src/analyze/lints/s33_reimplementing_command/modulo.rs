use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern: x - y * floor(x / y)
// This is a manual implementation of the modulo operation

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for subtraction: x - (...)
    let Expression::BinaryCommand(BinaryCommand::Sub, sub_lhs, sub_rhs, _) = target else {
        return codes;
    };

    // Check for multiplication on the right: y * floor(x / y)
    let Expression::BinaryCommand(BinaryCommand::Mul, mul_lhs, mul_rhs, _) = &**sub_rhs else {
        return codes;
    };

    // Check for floor unary command
    let Expression::UnaryCommand(UnaryCommand::Named(floor_cmd), floor_arg, _) = &**mul_rhs else {
        return codes;
    };

    if !floor_cmd.eq_ignore_ascii_case("floor") {
        return codes;
    }

    // Check for division inside floor: x / y
    let Expression::BinaryCommand(BinaryCommand::Div, div_lhs, div_rhs, _) = &**floor_arg else {
        return codes;
    };

    // Check if variables match the pattern: x - y * floor(x / y)
    // x should match in sub_lhs and div_lhs
    // y should match in mul_lhs and div_rhs
    if super::expressions_match(sub_lhs, div_lhs, false)
        && super::expressions_match(mul_lhs, div_rhs, false)
    {
        let left_text = sub_lhs.source(true);
        let right_text = mul_lhs.source(true);
        codes.push(Arc::new(CodeS33ReimplementingCommandMod::new(
            target.full_span(),
            left_text,
            right_text,
            processed,
            config.severity(),
        )));
    }

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandMod {
    span: Range<usize>,
    left: String,
    right: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandMod {
    fn ident(&self) -> &'static str {
        "L-S33-MOD"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `mod`")
    }

    fn label_message(&self) -> String {
        String::from("use `mod`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} mod {}", self.left, self.right))
    }
}

impl CodeS33ReimplementingCommandMod {
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
