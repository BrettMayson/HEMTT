use std::sync::Arc;

use float_ord::FloatOrd;
use hemtt_workspace::reporting::{Code, Processed};

use crate::{Expression, NularCommand, Statements, UnaryCommand};

use super::codes::saa3_typename::Typename;

pub fn typename(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
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
    if name.to_lowercase() != "typename" {
        return Vec::new();
    }
    let target_span = target.span();
    let (constant_type, span, length) = match &**target {
        Expression::String(s, span, _) => ("STRING", span, s.len() + 2),
        Expression::Number(FloatOrd(s), span) => ("SCALAR", span, s.to_string().len()),
        Expression::Boolean(bool, span) => ("BOOL", span, bool.to_string().len()),
        Expression::Code(statements) if statements.content().is_empty() => {
            ("CODE", &target_span, statements.span().len())
        }
        Expression::Array(array, span) if array.is_empty() => {
            ("ARRAY", &target_span, span.len().max(2))
        }
        Expression::NularCommand(NularCommand { name }, span) => {
            let (a, b) = match name.as_str() {
                "scriptnull" => ("SCRIPT", span),
                "objnull" => ("OBJECT", span),
                "grpnull" => ("GROUP", span),
                "controlnull" => ("CONTROL", span),
                "teammembernull" => ("TEAM_MEMBER", span),
                "displaynull" => ("DISPLAY", span),
                "tasknull" => ("TASK", span),
                "locationnull" => ("LOCATION", span),
                "sideunknown" => ("SIDE", span),
                "configfile" | "confignull" => ("CONFIG", span),
                "missionnamespace" | "profilenamespace" | "uinamespace" | "parsingnamespace" => {
                    ("NAMESPACE", span)
                }
                "diaryrecordnull" => ("DIARY_RECORD", span),
                "createhashmap" => ("HASHMAP", span),
                _ => return Vec::new(),
            };
            (a, b, name.len())
        }
        Expression::UnaryCommand(UnaryCommand::Named(name), _, span) => {
            if name == "text" {
                ("TEXT", span, name.len())
            } else {
                return Vec::new();
            }
        }
        _ => return Vec::new(),
    };
    vec![Arc::new(Typename::new(
        expression.full_span(),
        (constant_type.to_string(), span.clone(), length),
        processed,
    ))]
}
