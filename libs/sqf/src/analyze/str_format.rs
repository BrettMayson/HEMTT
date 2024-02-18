use std::sync::Arc;

use hemtt_common::reporting::{Code, Processed};

use crate::{analyze::codes::saa4_str_format::StrFormat, Expression, Statements, UnaryCommand};

pub fn str_format(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed));
        }
    }
    advice
}

fn check_expression(expression: &Expression, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let Expression::UnaryCommand(UnaryCommand::Named(name), target, _) = expression else {
        return Vec::new();
    };
    if name.to_lowercase() != "format" {
        return Vec::new();
    }
    let Expression::Array(args, _) = &**target else {
        return Vec::new();
    };
    if args.len() != 2 {
        return Vec::new();
    }
    let Expression::String(format, _) = &args[0] else {
        return Vec::new();
    };
    if &**format != "%1" {
        return Vec::new();
    }
    StrFormat::new(expression.full_span(), args[1].clone(), processed)
        .map_or_else(Vec::new, |code| vec![Arc::new(code) as Arc<dyn Code>])
}
