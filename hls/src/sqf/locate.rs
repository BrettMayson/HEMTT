use hemtt_sqf::{Expression, Statement, Statements};
use hemtt_workspace::reporting::Processed;
use tower_lsp::lsp_types::Position;
use url::Url;

use super::SqfCache;

pub trait Locate {
    fn locate_expression(&self, uri: Url, position: Position) -> Option<Expression>;
}

impl Locate for (&Processed, &Statements) {
    fn locate_expression(&self, uri: Url, position: Position) -> Option<Expression> {
        SqfCache::get().files.read().unwrap().get(&uri).map(
            |(processed, workspace_path, statements, _)| {
                let offset = processed
                    .line_offset(workspace_path, position.line as usize)
                    .unwrap_or_default()
                    + position.character as usize;
                for statement in statements.content().iter() {
                    match statement {
                        Statement::AssignGlobal(_, expression, _)
                        | Statement::AssignLocal(_, expression, _)
                        | Statement::Expression(expression, _) => {
                            if let Some(exp) = locate_expression(self.0, expression, offset) {
                                return Some(exp);
                            }
                        }
                    }
                }
                None
            },
        )?
    }
}

fn locate_expression(
    processed: &Processed,
    expression: &Expression,
    offset: usize,
) -> Option<Expression> {
    let start_map = processed.mapping(expression.full_span().start)?;
    let end_map = processed.mapping(expression.full_span().end)?;
    if start_map.original_start() >= offset || end_map.original_end() <= offset {
        return None;
    }
    match expression {
        Expression::Code(statements) => {
            for statement in statements.content().iter() {
                match statement {
                    Statement::AssignGlobal(_, expression, _)
                    | Statement::AssignLocal(_, expression, _)
                    | Statement::Expression(expression, _) => {
                        if let Some(exp) = locate_expression(processed, expression, offset) {
                            return Some(exp);
                        }
                    }
                }
            }
            None
        }
        Expression::String(_, span, _)
        | Expression::Number(_, span)
        | Expression::Variable(_, span)
        | Expression::Boolean(_, span) => {
            let map = processed.mapping(span.start)?;
            if map.original_start() <= offset && map.original_end() > offset {
                Some(expression.clone())
            } else {
                None
            }
        }
        Expression::Array(experssions, _) => {
            for expression in experssions.iter() {
                if let Some(exp) = locate_expression(processed, expression, offset) {
                    return Some(exp);
                }
            }
            None
        }
        Expression::NularCommand(_, _) => Some(expression.clone()),
        Expression::UnaryCommand(_, lhs, _) => {
            let start_map = processed.mapping(lhs.full_span().start)?;
            let end_map = processed.mapping(lhs.full_span().end)?;
            if start_map.original_start() <= offset && end_map.original_end() > offset {
                locate_expression(processed, lhs, offset)
            } else {
                Some(expression.clone())
            }
        }
        Expression::BinaryCommand(_, lhs, rhs, _) => {
            for hs in [lhs, rhs].iter() {
                let start_map = processed.mapping(hs.full_span().start)?;
                let end_map = processed.mapping(hs.full_span().end)?;
                if start_map.original_start() <= offset && end_map.original_end() > offset {
                    return locate_expression(processed, hs, offset);
                }
            }
            Some(expression.clone())
        }
    }
}
