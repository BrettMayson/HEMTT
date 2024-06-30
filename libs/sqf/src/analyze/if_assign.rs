use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Processed};

use crate::{
    analyze::codes::saa1_if_assign::IfAssign, BinaryCommand, Expression, Statements, UnaryCommand,
};

use super::extract_constant;

pub fn if_assign(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed));
        }
    }
    advice
}

fn check_expression(expression: &Expression, processed: &Processed) -> Vec<Arc<dyn Code>> {
    if let Expression::BinaryCommand(BinaryCommand::Named(name), if_cmd, code, _) = expression {
        if name.to_lowercase() == "then" {
            let Expression::UnaryCommand(UnaryCommand::Named(_), condition, _) = &**if_cmd else {
                return Vec::new();
            };
            if let Expression::BinaryCommand(BinaryCommand::Else, lhs_expr, rhs_expr, _) = &**code {
                let lhs = extract_constant(lhs_expr);
                let rhs = extract_constant(rhs_expr);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    // Skip if consts are used in a isNil check (e.g. [x, 5] select (isNil "x") will error in scheduled)
                    if let Expression::UnaryCommand(UnaryCommand::Named(name), _, _) = &**condition
                    {
                        if name.to_lowercase() == "isnil" {
                            return Vec::new();
                        }
                    }
                    return vec![Arc::new(IfAssign::new(
                        if_cmd.span(),
                        (condition.source(), condition.full_span()),
                        (lhs, lhs_expr.span()),
                        (rhs, rhs_expr.span()),
                        processed,
                    ))];
                }
            }
        }
    }
    Vec::new()
}
