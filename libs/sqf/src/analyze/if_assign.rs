use std::sync::Arc;

use hemtt_workspace::reporting::{Code, Processed};

use crate::{
    analyze::codes::saa1_if_assign::IfAssign, BinaryCommand, Expression, Statement, Statements,
    UnaryCommand,
};

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
        if name == "then" {
            let Expression::UnaryCommand(UnaryCommand::Named(_), condition, _) = &**if_cmd else {
                return Vec::new();
            };
            if let Expression::BinaryCommand(BinaryCommand::Else, lhs_expr, rhs_expr, _) = &**code {
                let lhs = extract_constant(lhs_expr);
                let rhs = extract_constant(rhs_expr);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
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

/// Extracts a constant from an expression
///
/// Returns a tuple of the constant and a boolean indicating if quotes are needed
fn extract_constant(expression: &Expression) -> Option<(String, bool)> {
    if let Expression::Code(code) = &expression {
        if code.content.len() == 1 {
            if let Statement::Expression(expr, _) = &code.content[0] {
                return match expr {
                    Expression::Boolean(bool, _) => Some((bool.to_string(), false)),
                    Expression::Number(num, _) => Some((num.0.to_string(), false)),
                    Expression::String(string, _) => Some((string.to_string(), true)),
                    Expression::Variable(var, _) => Some((var.to_string(), false)),
                    _ => None,
                };
            }
        }
    }
    None
}
