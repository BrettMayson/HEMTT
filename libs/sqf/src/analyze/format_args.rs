use crate::{analyze::codes::saa7_format_args::FormatArgs, Expression, Statements, UnaryCommand};
use hemtt_workspace::reporting::{Code, Processed};
use std::{cmp::Ordering, sync::Arc};

#[must_use]
pub fn format_args(statements: &Statements, processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut advice: Vec<Arc<dyn Code>> = Vec::new();
    for statement in statements.content() {
        for expression in statement.walk_expressions() {
            advice.extend(check_expression(expression, processed));
        }
    }
    advice
}

#[must_use]
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
    if args.is_empty() {
        return vec![Arc::new(FormatArgs::new(
            expression.full_span(),
            "format string: empty array".to_string(),
            processed,
        ))];
    }
    let Expression::String(format, _, _) = &args[0] else {
        return Vec::new();
    };

    #[allow(clippy::option_if_let_else)]
    if let Some(problem) = get_format_problem(format, args.len() - 1) {
        vec![Arc::new(FormatArgs::new(
            expression.full_span(),
            problem,
            processed,
        ))]
    } else {
        Vec::new()
    }
}

#[must_use]
fn get_format_problem(input: &str, extra_args: usize) -> Option<String> {
    let format = format!("{input} ",); // add extra terminator

    let mut tokens: Vec<usize> = Vec::new();
    let mut token_active = false;
    let mut token_start = 0;
    for (i, c) in format.chars().enumerate() {
        if token_active && !c.is_ascii_digit() {
            token_active = false;
            if i > token_start {
                let token_value = format
                    .chars()
                    .take(i)
                    .skip(token_start)
                    .collect::<String>()
                    .parse()
                    .unwrap_or_default();
                tokens.push(token_value);
            } else if c != '%' {
                return Some(format!(
                    "format string: non-escaped \"%\" [at index {token_start}]"
                ));
            }
        }
        if !token_active && c == '%' {
            token_active = true;
            token_start = i + 1;
        }
    }
    let max_index = *tokens.iter().max().unwrap_or(&0);

    match extra_args.cmp(tokens.iter().max().unwrap_or(&0)) {
        Ordering::Less => Some(format!(
            "format string: undefined tokens [used \"%{max_index}\", passed {extra_args}]"
        )),
        Ordering::Greater => Some(format!(
            "format string: unused args [used \"%{max_index}\", passed {extra_args}]"
        )),
        Ordering::Equal => {
            if max_index > tokens.len() {
                Some(format!(
                    "format string: skipped tokens [used \"%{max_index}\", but only {} tokens]",
                    tokens.len()
                ))
            } else {
                None
            }
        }
    }
}

#[test]
fn test() {
    assert!(get_format_problem("", 0).is_none());
    assert!(get_format_problem("%1%2", 2).is_none());
    assert!(get_format_problem("🌭%1", 1).is_none());
    assert!(get_format_problem("%1%2", 1).is_some()); // undefined tokens
    assert!(get_format_problem("%1%2", 3).is_some()); // unused args
    assert!(get_format_problem("%2", 2).is_some()); // skipped tokens
    assert!(get_format_problem("50%", 0).is_some()); // un-escaped %
}
