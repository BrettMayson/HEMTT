use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Processed};

use crate::{parser::database::Database, Expression, Statements};

use super::codes::saa6_command_case::CommandCase;

pub fn command_case(
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
    let Some(command) = expression.command_name() else {
        return Vec::new();
    };
    let Some(wiki) = database.wiki().commands().get(&command.to_lowercase()) else {
        return Vec::new();
    };
    if command != wiki.name() {
        return vec![Arc::new(CommandCase::new(
            expression.span(),
            command.to_string(),
            wiki.name().to_string(),
            processed,
        ))];
    }
    Vec::new()
}
