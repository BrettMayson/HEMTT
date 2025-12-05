use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression};

// Pattern 1: x + (1 - (x % 1))
// Pattern 2: (x - (x % 1)) + 1
// These are common patterns to implement ceil function

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for addition
    let Expression::BinaryCommand(BinaryCommand::Add, add_lhs, add_rhs, _) = target else {
        return codes;
    };

    // Pattern 1: x + (1 - (x % 1))
    if let Some(var_expr) = check_pattern1(add_lhs, add_rhs).or_else(|| check_pattern1(add_rhs, add_lhs)) {
        let var_text = var_expr.source(true);
        codes.push(Arc::new(CodeS33ReimplementingCommandCeil::new(
            target.full_span(),
            var_text,
            processed,
            config.severity(),
        )));
        return codes;
    }

    // Pattern 2: (x - (x % 1)) + 1
    if let Some(var_expr) = check_pattern2(add_lhs, add_rhs).or_else(|| check_pattern2(add_rhs, add_lhs)) {
        let var_text = var_expr.source(true);
        codes.push(Arc::new(CodeS33ReimplementingCommandCeil::new(
            target.full_span(),
            var_text,
            processed,
            config.severity(),
        )));
    }

    codes
}

// Pattern 1: x + (1 - (x % 1))
fn check_pattern1<'a>(var_expr: &'a Expression, sub_expr: &'a Expression) -> Option<&'a Expression> {
    // Check if sub_expr is (1 - (x % 1))
    let Expression::BinaryCommand(BinaryCommand::Sub, sub_lhs, sub_rhs, _) = sub_expr else {
        return None;
    };

    // Check if left side is 1
    let Expression::Number(num, _) = &**sub_lhs else {
        return None;
    };

    if (num.0 - 1.0).abs() > f32::EPSILON {
        return None;
    }

    // Check if right side is (x % 1)
    let Expression::BinaryCommand(BinaryCommand::Rem, mod_lhs, mod_rhs, _) = &**sub_rhs else {
        return None;
    };

    // Check if modulo is with 1
    let Expression::Number(mod_num, _) = &**mod_rhs else {
        return None;
    };

    if (mod_num.0 - 1.0).abs() > f32::EPSILON {
        return None;
    }

    // Check if the variable matches
    if super::expressions_match(var_expr, mod_lhs, false) {
        Some(var_expr)
    } else {
        None
    }
}

// Pattern 2: (x - (x % 1)) + 1
fn check_pattern2<'a>(sub_expr: &'a Expression, one_expr: &'a Expression) -> Option<&'a Expression> {
    // Check if one_expr is 1
    let Expression::Number(num, _) = one_expr else {
        return None;
    };

    if (num.0 - 1.0).abs() > f32::EPSILON {
        return None;
    }

    // Check if sub_expr is (x - (x % 1))
    let Expression::BinaryCommand(BinaryCommand::Sub, sub_lhs, sub_rhs, _) = sub_expr else {
        return None;
    };

    // Check if right side is (x % 1)
    let Expression::BinaryCommand(BinaryCommand::Rem, mod_lhs, mod_rhs, _) = &**sub_rhs else {
        return None;
    };

    // Check if modulo is with 1
    let Expression::Number(mod_num, _) = &**mod_rhs else {
        return None;
    };

    if (mod_num.0 - 1.0).abs() > f32::EPSILON {
        return None;
    }

    // Check if both variables match
    if super::expressions_match(sub_lhs, mod_lhs, false) {
        Some(sub_lhs)
    } else {
        None
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandCeil {
    span: Range<usize>,
    var: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandCeil {
    fn ident(&self) -> &'static str {
        "L-S33-CEIL"
    }
    
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `ceil`")
    }

    fn label_message(&self) -> String {
        String::from("use `ceil`")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("ceil {}", self.var))
    }
}

impl CodeS33ReimplementingCommandCeil {
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
