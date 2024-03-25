use std::sync::Arc;

use float_ord::FloatOrd;
use hemtt_workspace::reporting::{Code, Processed};

use crate::{BinaryCommand, Expression, Statements, UnaryCommand};

use super::codes::saa2_find_in_str::FindInStr;

pub fn find_in_str(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed));
        }
    }
    advice
}

fn check_expression(expression: &Expression, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let Expression::BinaryCommand(BinaryCommand::Greater, search, compare, _) = expression else {
        return Vec::new();
    };
    let Expression::UnaryCommand(UnaryCommand::Minus, to, _) = &**compare else {
        return Vec::new();
    };
    let Expression::Number(FloatOrd(num), _) = &**to else {
        return Vec::new();
    };
    if (num - 1.0).abs() > std::f32::EPSILON {
        return Vec::new();
    }
    let Expression::BinaryCommand(BinaryCommand::Named(name), haystack, needle, _) = &**search
    else {
        return Vec::new();
    };
    if name.to_lowercase() != "find" {
        return Vec::new();
    }

    if let Expression::String(needle_str, _) = &**needle {
        let haystack_str = match &**haystack {
            Expression::String(s, _) => format!("\"{s}\""),
            Expression::Variable(name, _) => name.to_string(),
            _ => return Vec::new(),
        };
        return vec![Arc::new(FindInStr::new(
            haystack.span().start..expression.full_span().end,
            (haystack_str, haystack.span()),
            (format!("\"{needle_str}\""), needle.span()),
            processed,
        ))];
    }
    Vec::new()
}
