use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression};

// Pattern: x isEqualType a || x isEqualType b || ...
// SQF Structure: (binary ||) of two or more `isEqualType` comparisons sharing a left hand side

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(BinaryCommand::Or, left, _, _) = target else {
        return codes;
    };
    // `a || b || c` parses as `(a || b) || c`, and every `||` in the chain is visited.
    // Only the innermost is reported, so a chain produces one finding rather than one per link.
    if matches!(**left, Expression::BinaryCommand(BinaryCommand::Or, ..)) {
        return codes;
    }

    let mut branches = Vec::new();
    if !flatten_or(target, &mut branches) {
        return codes;
    }

    // every branch must be `x isEqualType <type>` with the same x
    let mut types = Vec::new();
    let mut value: Option<&Expression> = None;
    for branch in branches {
        let Some((lhs, rhs)) = as_is_equal_type(branch) else {
            return codes;
        };
        match value {
            None => value = Some(lhs),
            Some(previous) if super::expressions_match(previous, lhs, false) => {}
            Some(_) => return codes,
        }
        types.push(rhs.source(true));
    }

    let Some(value) = value else {
        return codes;
    };
    if types.len() < 2 {
        return codes;
    }

    codes.push(Arc::new(CodeS33ReimplementingCommandIsEqualTypeAny::new(
        target.full_span(),
        value.source(true),
        types,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Flattens a chain of `||` into its branches. Returns false if a branch is itself unusable.
fn flatten_or<'a>(expr: &'a Expression, out: &mut Vec<&'a Expression>) -> bool {
    if let Expression::BinaryCommand(BinaryCommand::Or, left, right, _) = expr {
        return flatten_or(left, out) && flatten_or(right, out);
    }
    out.push(expr);
    true
}

/// Matches `x isEqualType y`, seeing through a short circuit code block on the right of `||`.
fn as_is_equal_type(expr: &Expression) -> Option<(&Expression, &Expression)> {
    let expr = super::unwrap_code_block(expr);
    let Expression::BinaryCommand(BinaryCommand::Named(name), lhs, rhs, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("isEqualType") {
        return None;
    }
    Some((lhs, rhs))
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandIsEqualTypeAny {
    span: Range<usize>,
    value: String,
    types: Vec<String>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandIsEqualTypeAny {
    fn ident(&self) -> &'static str {
        "L-S33-ISEQUALTYPEANY"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `isEqualTypeAny`")
    }

    fn label_message(&self) -> String {
        String::from("use `isEqualTypeAny`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{} isEqualTypeAny [{}]",
            self.value,
            self.types.join(", ")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandIsEqualTypeAny {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        value: String,
        types: Vec<String>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            value,
            types,
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
