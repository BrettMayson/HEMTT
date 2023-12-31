use std::sync::Arc;

use float_ord::FloatOrd;
use hemtt_common::reporting::{Code, Processed};

use crate::{
    analyze::codes::saa5_select_parse_number::SelectParseNumber, BinaryCommand, Expression,
    Statements,
};

pub fn select_parse_number(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed));
        }
    }
    advice
}

fn check_expression(expression: &Expression, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let Expression::BinaryCommand(BinaryCommand::Named(name), target, condition, _) = expression
    else {
        return Vec::new();
    };
    if name.to_lowercase() != "select" {
        return Vec::new();
    }
    let Expression::Array(args, _) = &**target else {
        return Vec::new();
    };
    if args.len() != 2 {
        return Vec::new();
    }
    let Expression::Number(FloatOrd(mut lhs), _) = &args[0] else {
        return Vec::new();
    };
    let Expression::Number(FloatOrd(mut rhs), _) = &args[1] else {
        return Vec::new();
    };
    let mut negate = false;
    if rhs.abs() < std::f32::EPSILON {
        negate = true;
        std::mem::swap(&mut lhs, &mut rhs);
    }
    if lhs.abs() > std::f32::EPSILON || (rhs - 1.0).abs() > std::f32::EPSILON {
        return Vec::new();
    }
    vec![Arc::new(SelectParseNumber::new(
        expression.full_span(),
        (**condition).clone(),
        processed,
        negate,
    ))]
}
