use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    WorkspacePath, lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Label, Processed, Severity, get_span_info},
};

use crate::{
    BinaryCommand, Expression, Statement, Statements, UnaryCommand,
    analyze::LintData,
};

crate::analyze::lint!(LintS40StringConcatInLoop);

impl Lint<LintData> for LintS40StringConcatInLoop {
    fn ident(&self) -> &'static str {
        "string_concat_in_loop"
    }

    fn sort(&self) -> u32 {
        400
    }

    fn description(&self) -> &'static str {
        "Checks for repeated string concatenation inside loops"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
private _myString = "";
for [{_i = 0}, {_i < 10000}, {_i = _i + 1}] do {
    _myString = _myString + "123";
};
```

**Correct**
```sqf
private _strings = [];
for [{_i = 0}, {_i < 10000}, {_i = _i + 1}] do {
    _strings pushBack "123";
};
private _myString = _strings joinString "";
```

### Explanation

Repeatedly concatenating a string inside a loop can become very slow for large iteration counts. Building an array of fragments and joining them once at the end is typically much faster.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_options(
            std::iter::once((
                "threshold".to_string(),
                toml::Value::Integer(1000),
            ))
            .collect(),
        )
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Statements;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };

        let threshold = config
            .option("threshold")
            .and_then(toml::Value::as_integer)
            .unwrap_or(1000);

        let mut codes: Codes = Vec::new();
        for (index, statement) in target.content().iter().enumerate() {
            let Some((loop_count, body)) = extract_loop_body(statement) else {
                continue;
            };
            if loop_count <= threshold {
                continue;
            }

            let Some((concat_var, concat_span)) = find_string_concat_assignment(body) else {
                continue;
            };
            if !is_string_accumulator(target.content(), index, &concat_var) {
                continue;
            }
            let Some(declaration_span) = find_variable_declaration_span(target.content(), index, &concat_var) else {
                continue;
            };

            codes.push(Arc::new(CodeS40StringConcatInLoop::new(
                Some(declaration_span),
                concat_span,
                processed,
                config.severity(),
            )));
        }

        codes
    }
}

fn extract_loop_body(statement: &Statement) -> Option<(i64, &Statements)> {
    let Statement::Expression(expression, _) = statement else {
        return None;
    };

    match expression {
        Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _) if cmd.eq_ignore_ascii_case("do") => {
            let loop_count = match lhs.as_ref() {
                Expression::UnaryCommand(UnaryCommand::Named(name), _, _) if name.eq_ignore_ascii_case("for") => {
                    parse_for_loop_iterations(lhs.as_ref())
                }
                Expression::UnaryCommand(UnaryCommand::Named(name), loop_args, _) if name.eq_ignore_ascii_case("foreach") => {
                    #[allow(clippy::cast_possible_wrap)]
                    let loop_count = match loop_args.as_ref() {
                        Expression::Array(values, _) | Expression::ConsumeableArray(values, _) => Some(values.len() as i64),
                        _ => None,
                    }?;
                    Some(loop_count)
                }
                _ => parse_for_loop_iterations(lhs.as_ref()),
            }?;

            let Expression::Code(body) = rhs.as_ref() else { return None };

            Some((loop_count, body))
        }
        Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _) if cmd.eq_ignore_ascii_case("foreach") || cmd.eq_ignore_ascii_case("forEach") => {
            let Expression::Code(body) = lhs.as_ref() else { return None };
            #[allow(clippy::cast_possible_wrap)]
            let loop_count = match rhs.as_ref() {
                Expression::Array(values, _) | Expression::ConsumeableArray(values, _) => values.len() as i64,
                _ => i64::MAX,
            };
            Some((loop_count, body))
        }
        _ => None,
    }
}

fn find_variable_declaration_span(statements: &[Statement], current_index: usize, variable: &str) -> Option<Range<usize>> {
    for statement in statements.iter().take(current_index).rev() {
        match statement {
            Statement::AssignGlobal(name, _, span) | Statement::AssignLocal(name, _, span) => {
                if name == variable {
                    return Some(span.clone());
                }
            }
            Statement::Expression(Expression::UnaryCommand(UnaryCommand::Named(command), rhs, _), _) if command.eq_ignore_ascii_case("private") => {
                if let Some((declared_var, span)) = collect_declared_variables(rhs).into_iter().find(|(declared_var, _)| declared_var == variable) {
                    let _ = declared_var;
                    return Some(span);
                }
            }
            Statement::Expression(_, _) => {},
        }
    }
    None
}

fn collect_declared_variables(expression: &Expression) -> Vec<(String, Range<usize>)> {
    match expression {
        Expression::String(name, span, _) if !name.is_empty() => vec![(name.to_string(), span.clone())],
        Expression::Array(values, _) => values
            .iter()
            .filter_map(|value| match value {
                Expression::String(name, span, _) if !name.is_empty() => Some((name.to_string(), span.clone())),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_for_loop_iterations(expression: &Expression) -> Option<i64> {
    match expression {
        Expression::UnaryCommand(UnaryCommand::Named(name), loop_args, _) if name.eq_ignore_ascii_case("for") => {
            if let Some(iterations) = parse_bracket_loop_iterations(loop_args.as_ref()) {
                return Some(iterations);
            }

            parse_from_to_loop_iterations(loop_args.as_ref())
        }
        Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) if command.eq_ignore_ascii_case("to") => {
            parse_from_to_loop_iterations(expression)
        }
        Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) if command.eq_ignore_ascii_case("from") => {
            parse_from_to_loop_iterations(expression)
        }
        _ => None,
    }
}

fn parse_bracket_loop_iterations(expression: &Expression) -> Option<i64> {
    let Expression::Array(parts, _) = expression else {
        return None;
    };
    if parts.len() != 3 {
        return None;
    }

    let Some(Expression::Code(Statements { content, .. })) = parts.first() else {
        return None;
    };
    let Some(Statement::AssignGlobal(_, init, _) | Statement::AssignLocal(_, init, _)) = content.first() else {
        return None;
    };
    #[allow(clippy::cast_possible_truncation)]
    let init_value = match init {
        Expression::Number(n, _) => n.0 as i64,
        _ => return None,
    };

    let Some(Expression::Code(Statements { content, .. })) = parts.get(1) else {
        return None;
    };
    let Some(Statement::Expression(Expression::BinaryCommand(command, lhs, rhs, _), _)) = content.first() else {
        return None;
    };
    #[allow(clippy::cast_possible_truncation)]
    let limit = match command {
        BinaryCommand::Less | BinaryCommand::LessEq => match rhs.as_ref() {
            Expression::Number(n, _) => n.0 as i64,
            _ => return None,
        },
        BinaryCommand::Greater | BinaryCommand::GreaterEq => match lhs.as_ref() {
            Expression::Number(n, _) => n.0 as i64,
            _ => return None,
        },
        _ => return None,
    };

    let Some(Expression::Code(Statements { content, .. })) = parts.get(2) else {
        return None;
    };
    let Some(Statement::AssignGlobal(_, increment, _) | Statement::AssignLocal(_, increment, _)) = content.first() else {
        return None;
    };
    #[allow(clippy::cast_possible_truncation)]
    let step = match increment {
        Expression::BinaryCommand(BinaryCommand::Add, _, rhs, _) => match rhs.as_ref() {
            Expression::Number(n, _) => n.0 as i64,
            _ => return None,
        },
        Expression::BinaryCommand(BinaryCommand::Sub, _, rhs, _) => match rhs.as_ref() {
            Expression::Number(n, _) => -(n.0 as i64),
            _ => return None,
        },
        _ => return None,
    };

    let span = limit - init_value;
    if span <= 0 {
        return Some(0);
    }
    Some(span / step)
}

fn parse_from_to_loop_iterations(expression: &Expression) -> Option<i64> {
    let Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) = expression else {
        return None;
    };
    if !command.eq_ignore_ascii_case("to") {
        return None;
    }

    let start_value = parse_from_expression(lhs.as_ref())?;
    #[allow(clippy::cast_possible_truncation)]
    let limit_value = match rhs.as_ref() {
        Expression::Number(n, _) => n.0 as i64,
        _ => return None,
    };

    let span = limit_value - start_value;
    if span <= 0 {
        return Some(0);
    }
    Some(span)
}

fn parse_from_expression(expression: &Expression) -> Option<i64> {
    let Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) = expression else {
        return None;
    };
    if !command.eq_ignore_ascii_case("from") {
        return None;
    }

    let _ = lhs.as_ref();
    #[allow(clippy::cast_possible_truncation)]
    match rhs.as_ref() {
        Expression::Number(n, _) => Some(n.0 as i64),
        _ => None,
    }
}

fn find_string_concat_assignment(statements: &Statements) -> Option<(String, Range<usize>)> {
    statements.content().iter().find_map(|statement| match statement {
        Statement::AssignGlobal(var, expr, span) | Statement::AssignLocal(var, expr, span) => {
            is_string_concat_assignment(var, expr).then(|| (var.clone(), span.clone()))
        }
        Statement::Expression(_, _) => None,
    })
}

fn is_string_accumulator(statements: &[Statement], current_index: usize, variable: &str) -> bool {
    for statement in statements.iter().take(current_index).rev() {
        match statement {
            Statement::AssignGlobal(name, expr, _) | Statement::AssignLocal(name, expr, _) if name == variable => {
                return is_string_initializer(expr);
            }
            Statement::Expression(Expression::UnaryCommand(UnaryCommand::Named(command), rhs, _), _) if command.eq_ignore_ascii_case("private")
                && collect_declared_variables(rhs)
                    .into_iter()
                    .any(|(declared_var, _)| declared_var == variable)
                => {
                    return false;
                }
            _ => {}
        }
    }
    false
}

fn is_string_initializer(expression: &Expression) -> bool {
    match expression {
        Expression::String(_, _, _) => true,
        Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) => {
            is_string_initializer(lhs.as_ref()) || is_string_initializer(rhs.as_ref())
        }
        _ => false,
    }
}

fn is_string_concat_assignment(variable: &str, expression: &Expression) -> bool {
    let Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) = expression else {
        return false;
    };
    let lhs_is_variable = matches!(lhs.as_ref(), Expression::Variable(name, _) if name == variable);
    let rhs_is_variable = matches!(rhs.as_ref(), Expression::Variable(name, _) if name == variable);
    if !lhs_is_variable && !rhs_is_variable {
        return false;
    }
    is_string_like(lhs.as_ref()) || is_string_like(rhs.as_ref())
}

fn is_string_like(expression: &Expression) -> bool {
    match expression {
        Expression::String(_, _, _) | Expression::Variable(_, _) => true,
        Expression::UnaryCommand(UnaryCommand::Named(name), _, _) => {
            name.eq_ignore_ascii_case("format")
        }
        Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) => {
            is_string_like(lhs.as_ref()) || is_string_like(rhs.as_ref())
        }
        _ => false,
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS40StringConcatInLoop {
    declaration_span: Option<Range<usize>>,
    concat_span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS40StringConcatInLoop {
    fn ident(&self) -> &'static str {
        "L-S40"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#string_concat_in_loop")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "String concatenation inside a loop can become slow; use an array and joinString instead".to_string()
    }

    fn label_message(&self) -> String {
        "expensive string concatenation in loop".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("use an array of fragments and `joinString` after the loop".to_string())
    }

    fn note(&self) -> Option<String> {
        Some("If the array being iterated is large, this can become slow".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS40StringConcatInLoop {
    #[must_use]
    pub fn new(
        declaration_span: Option<Range<usize>>,
        concat_span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            declaration_span,
            concat_span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.concat_span.clone(), processed) else {
            return self;
        };
        diag = diag.clear_labels();
        if let Some(span) = &self.declaration_span
            && let Some((file, span)) = get_span_info(span, processed) {
                diag = diag.with_label(Label::secondary(file, span).with_message("accumulator declaration"));
            }
        if let Some((file, span)) = get_span_info(&self.concat_span, processed) {
            diag = diag.with_label(Label::primary(file, span).with_message("string concatenation in loop"));
        }
        self.diagnostic = Some(diag);
        self
    }
}
