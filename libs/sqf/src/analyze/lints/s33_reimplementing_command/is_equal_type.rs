use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern: typeName A == typeName B  ->  A isEqualType B
// The negated form maps to !(A isEqualType B).
// Comparing typeName results builds a string per side, isEqualType compares the types directly.

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(cmd, left, right, _) = target else {
        return codes;
    };
    let negated = match cmd {
        BinaryCommand::Eq => false,
        BinaryCommand::NotEq => true,
        BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isEqualTo") => false,
        BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isNotEqualTo") => true,
        _ => return codes,
    };
    let (Some(lhs), Some(rhs)) = (as_typename(left), as_typename(right)) else {
        return codes;
    };

    codes.push(Arc::new(CodeS33ReimplementingCommandIsEqualType::new(
        target.full_span(),
        lhs.source(true),
        rhs.source(true),
        negated,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Matches `typeName X`, returning X.
fn as_typename(expr: &Expression) -> Option<&Expression> {
    let Expression::UnaryCommand(UnaryCommand::Named(name), inner, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("typeName") {
        return None;
    }
    Some(inner)
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandIsEqualType {
    span: Range<usize>,
    left: String,
    right: String,
    negated: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandIsEqualType {
    fn ident(&self) -> &'static str {
        "L-S33-ISEQUALTYPE"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `isEqualType`")
    }

    fn label_message(&self) -> String {
        String::from("use `isEqualType`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(if self.negated {
            format!("!({} isEqualType {})", self.left, self.right)
        } else {
            format!("{} isEqualType {}", self.left, self.right)
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandIsEqualType {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        left: String,
        right: String,
        negated: bool,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            left,
            right,
            negated,
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
