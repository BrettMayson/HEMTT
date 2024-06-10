use std::sync::Arc;

use arma3_wiki::model::Value;
use float_ord::FloatOrd;
use hemtt_workspace::reporting::{Code, Processed};

use crate::{
    analyze::codes::saa5_select_parse_number::SelectParseNumber, parser::database::Database,
    BinaryCommand, Expression, Statements,
};

pub fn select_parse_number(
    statements: &Statements,
    processed: &Processed,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed, database));
        }
    }
    advice
}

fn check_expression(
    expression: &Expression,
    processed: &Processed,
    database: &Database,
) -> Vec<Arc<dyn Code>> {
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
    if !(match &**condition {
        Expression::Code(_)
        | Expression::Number(_, _)
        | Expression::Array(_, _)
        | Expression::Variable(_, _) => false,
        Expression::String(_, _, _) | Expression::Boolean(_, _) => true,
        Expression::NularCommand(cmd, _) => safe_command(cmd.as_str(), database),
        Expression::UnaryCommand(cmd, _, _) => safe_command(cmd.as_str(), database),
        Expression::BinaryCommand(cmd, _, _, _) => safe_command(cmd.as_str(), database),
    }) {
        return Vec::new();
    }
    let mut negate = false;
    if rhs.abs() < f32::EPSILON {
        negate = true;
        std::mem::swap(&mut lhs, &mut rhs);
    }
    if lhs.abs() > f32::EPSILON || (rhs - 1.0).abs() > f32::EPSILON {
        return Vec::new();
    }
    vec![Arc::new(SelectParseNumber::new(
        expression.full_span(),
        (**condition).clone(),
        processed,
        negate,
    ))]
}

fn safe_command(command: &str, database: &Database) -> bool {
    if let "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" = command {
        return true;
    }
    let Some(cmd) = database.wiki().commands().get(command) else {
        return false;
    };
    cmd.syntax().iter().all(|s| match &s.ret().0 {
        Value::Boolean | Value::String => true,
        Value::OneOf(rets) => rets
            .iter()
            .all(|(r, _)| matches!(r, Value::String | Value::Boolean)),
        _ => false,
    })
}
