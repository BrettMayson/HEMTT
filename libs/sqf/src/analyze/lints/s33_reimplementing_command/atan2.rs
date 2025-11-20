use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern: atan(y / x) or similar arctangent calculations
// This is simpler - atan2 handles edge cases better
// We'll detect: atan (y / x) or atan (y/x)

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for atan unary command
    let Expression::UnaryCommand(UnaryCommand::Named(atan_cmd), atan_arg, _) = target else {
        return codes;
    };

    if !atan_cmd.eq_ignore_ascii_case("atan") {
        return codes;
    }

    // Check for division inside atan: y / x
    let Expression::BinaryCommand(BinaryCommand::Div, div_lhs, div_rhs, _) = &**atan_arg else {
        return codes;
    };

    // If we have atan(y / x), suggest using atan2 instead
    let left_text = div_lhs.source(true);
    let right_text = div_rhs.source(true);
    codes.push(Arc::new(CodeS33ReimplementingCommandAtan2::new(
        target.full_span(),
        left_text,
        right_text,
        processed,
        config.severity(),
    )));

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandAtan2 {
    span: Range<usize>,
    left: String,
    right: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandAtan2 {
    fn ident(&self) -> &'static str {
        "L-S33-ATAN2"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `atan2`")
    }

    fn label_message(&self) -> String {
        String::from("use `atan2`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} atan2 {}", self.left, self.right))
    }
}

impl CodeS33ReimplementingCommandAtan2 {
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
