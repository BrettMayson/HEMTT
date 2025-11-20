use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::reporting::{Code, Diagnostic, Processed, Severity};

use crate::{BinaryCommand, Expression, UnaryCommand};

// Pattern 1: sqrt((x1-x2)^2 + (y1-y2)^2) -> distance2D or distanceSqr if no sqrt
// Pattern 2: sqrt((x1-x2)^2 + (y1-y2)^2 + (z1-z2)^2) -> distance or distanceSqr if no sqrt

pub fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    // Check for sqrt unary command (highest priority - replaces most code)
    if let Expression::UnaryCommand(UnaryCommand::Named(sqrt_cmd), sqrt_arg, _) = target
        && sqrt_cmd.eq_ignore_ascii_case("sqrt")
    {
        // Check if it's a distance pattern with sqrt
        if let Some(dimension) = check_distance_pattern(sqrt_arg) {
            codes.push(Arc::new(CodeS33ReimplementingCommandDistance::new(
                target.full_span(),
                dimension,
                false,
                processed,
                config.severity(),
            )) as Arc<dyn Code>);
        }
        // Always return after checking sqrt - don't report nested patterns
        return codes;
    }

    // Only check for distanceSqr pattern (without sqrt) if this expression
    // is not the direct argument to a sqrt call OR part of a larger distance pattern
    // We detect this by checking if "sqrt(" appears right before our span
    // or if "+ (" appears right after (indicating this 2D pattern is part of a 3D pattern)
    if let Some(dimension) = check_distance_pattern(target) {
        let span = target.full_span();
        if !is_inside_sqrt_call(processed, span.start) 
            && !is_part_of_larger_pattern(processed, span.end, dimension) {
            codes.push(Arc::new(CodeS33ReimplementingCommandDistance::new(
                span,
                dimension,
                true,
                processed,
                config.severity(),
            )) as Arc<dyn Code>);
        }
    }

    codes
}

// Returns Some(2) for 2D, Some(3) for 3D, None if not a distance pattern
fn check_distance_pattern(expr: &Expression) -> Option<u8> {
    // Check for addition at the top level: (x1-x2)^2 + (y1-y2)^2 [+ (z1-z2)^2]
    let Expression::BinaryCommand(BinaryCommand::Add, add_lhs, add_rhs, _) = expr else {
        return None;
    };

    // Check if left side is another addition (3D case) or a squared difference (2D case)
    if let Expression::BinaryCommand(BinaryCommand::Add, inner_lhs, inner_rhs, _) = &**add_lhs {
        // 3D case: ((x1-x2)^2 + (y1-y2)^2) + (z1-z2)^2
        if is_squared_difference(inner_lhs)
            && is_squared_difference(inner_rhs)
            && is_squared_difference(add_rhs)
            && has_position_indicator(inner_lhs)
        {
            return Some(3);
        }
    }

    // 2D case: (x1-x2)^2 + (y1-y2)^2
    if is_squared_difference(add_lhs) && is_squared_difference(add_rhs) && has_position_indicator(add_lhs) {
        return Some(2);
    }

    None
}

// Check if a 2D pattern is part of a larger 3D pattern by looking ahead for "+ (..."
fn is_part_of_larger_pattern(processed: &Processed, end: usize, dimension: u8) -> bool {
    // Only 2D patterns can be part of a larger 3D pattern
    if dimension != 2 {
        return false;
    }
    
    let source = processed.as_str();
    if end >= source.len() {
        return false;
    }
    
    // Look ahead a bit to see if there's another "+ (" or "+(" suggesting continuation
    let search_end = (end + 20).min(source.len());
    let following = &source[end..search_end];
    
    // Look for patterns like: )  +  ( or ) + ( indicating another term being added
    let trimmed = following.trim_start();
    trimmed.starts_with('+') && trimmed[1..].trim_start().starts_with('(')
}

// Check if this expression is the direct argument to a sqrt call
// by looking backwards in the source for "sqrt("
fn is_inside_sqrt_call(processed: &Processed, start: usize) -> bool {
    let source = processed.as_str();
    
    // Look backwards from the start position for "sqrt("
    // We need to skip any whitespace/parens that might be between sqrt( and our expression
    if start < 5 {
        return false;
    }
    
    // Check a reasonable range before the expression (up to 20 chars for whitespace/parens)
    let search_start = start.saturating_sub(20);
    let preceding = &source[search_start..start];
    
    // Look for sqrt( pattern with possible whitespace/parens in between
    // Match patterns like: sqrt((, sqrt( (, etc.
    preceding.trim_end().ends_with("sqrt(")
        || preceding.trim_end_matches('(').trim_end().ends_with("sqrt")
}

// Check if expression contains indicators that suggest position data (not arbitrary scalars)
// This helps avoid false positives like angle calculations: sqrt((angle1-angle2)^2 + (angle3-angle4)^2)
fn has_position_indicator(expr: &Expression) -> bool {
    match expr {
        // Look for select operations which are common in position access: pos select 0
        Expression::BinaryCommand(BinaryCommand::Select, _, _, _) => true,
        Expression::BinaryCommand(BinaryCommand::Named(cmd), _, _, _) if cmd.eq_ignore_ascii_case("select") => true,
        // Recursively check subexpressions
        Expression::BinaryCommand(_, left, right, _) => {
            has_position_indicator(left) || has_position_indicator(right)
        }
        Expression::UnaryCommand(_, child, _) => has_position_indicator(child),
        Expression::Array(elements, _) => elements.iter().any(has_position_indicator),
        _ => false,
    }
}

// Check if expression is of form (a - b)^2
fn is_squared_difference(expr: &Expression) -> bool {
    // Check for exponentiation: (...) ^ 2
    let Expression::BinaryCommand(BinaryCommand::Exp, exp_lhs, exp_rhs, _) = expr else {
        return false;
    };

    // Check if exponent is 2
    let Expression::Number(num, _) = &**exp_rhs else {
        return false;
    };

    if (num.0 - 2.0).abs() > f32::EPSILON {
        return false;
    }

    // Check if base is a subtraction
    matches!(**exp_lhs, Expression::BinaryCommand(BinaryCommand::Sub, _, _, _))
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS33ReimplementingCommandDistance {
    span: Range<usize>,
    dimension: u8,
    is_sqr: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS33ReimplementingCommandDistance {
    fn ident(&self) -> &'static str {
        if self.is_sqr {
            "L-S33-DISTANCESQR"
        } else if self.dimension == 2 {
            "L-S33-DISTANCE2D"
        } else {
            "L-S33-DISTANCE"
        }
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#reimplementing_command")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        if self.is_sqr {
            String::from("code can be replaced with `distanceSqr`")
        } else if self.dimension == 2 {
            String::from("code can be replaced with `distance2D`")
        } else {
            String::from("code can be replaced with `distance`")
        }
    }

    fn label_message(&self) -> String {
        if self.is_sqr {
            String::from("use `distanceSqr`")
        } else if self.dimension == 2 {
            String::from("use `distance2D`")
        } else {
            String::from("use `distance`")
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS33ReimplementingCommandDistance {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        dimension: u8,
        is_sqr: bool,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            dimension,
            is_sqr,
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
