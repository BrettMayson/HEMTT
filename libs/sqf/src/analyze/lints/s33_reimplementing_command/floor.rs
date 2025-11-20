use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression};

// Pattern: x - (x % 1)
// This is a common pattern to implement floor function

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for subtraction
    let Expression::BinaryCommand(BinaryCommand::Sub, sub_lhs, sub_rhs, _) = target else {
        return codes;
    };

    // Check for modulo on the right side
    let Expression::BinaryCommand(BinaryCommand::Rem, mod_lhs, mod_rhs, _) = &**sub_rhs else {
        return codes;
    };

    // Check if modulo is with 1
    let Expression::Number(num, _) = &**mod_rhs else {
        return codes;
    };

    if (num.0 - 1.0).abs() > f32::EPSILON {
        return codes;
    }

    // Check if the variable in subtraction matches the variable in modulo
    if !super::expressions_match(sub_lhs, mod_lhs, false) {
        return codes;
    }

    let var_text = sub_lhs.source(true);
    codes.push(Arc::new(CodeS33ReimplementingCommandFloor::new(
        target.full_span(),
        var_text,
        processed,
        config.severity(),
    )));

    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandFloor {
    span: Range<usize>,
    var: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandFloor {
    fn ident(&self) -> &'static str {
        "L-S33-FLOOR"
    }
    
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `floor`")
    }

    fn label_message(&self) -> String {
        String::from("use `floor`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("floor {}", self.var))
    }
}

impl CodeS33ReimplementingCommandFloor {
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
