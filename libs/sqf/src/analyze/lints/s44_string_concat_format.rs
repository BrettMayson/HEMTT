use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{
    BinaryCommand, Expression, Statement, analyze::LintData,
};

crate::analyze::lint!(LintS44StringConcatFormat);

impl Lint<LintData> for LintS44StringConcatFormat {
    fn ident(&self) -> &'static str {
        "string_concat_format"
    }

    fn sort(&self) -> u32 {
        440
    }

    fn description(&self) -> &'static str {
        "Checks for string concatenation chains and suggests `format`"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
private _person = _first + " " + _last + ", " + str _age + " years old";
```

**Correct**
```sqf
private _person = format ["%1 %2, %3 years old", _first, _last, _age];
```

### Explanation

Chaining string concatenation with `+` can be harder to read and maintain than using `format`, especially when values are mixed with literals.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help().with_enabled(hemtt_common::config::LintEnabled::Disabled)
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Statement;

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

        let expression = match target {
            Statement::Expression(expression, _) => expression,
            Statement::AssignLocal(_, expression, _) | Statement::AssignGlobal(_, expression, _) => {
                expression
            }
        };

        let Some((_, best_expr)) = find_best_concat_chain(expression) else {
            return Vec::new();
        };

        let mut parts = Vec::new();
        collect_concat_parts(best_expr, &mut parts);
        if parts.len() < 3 {
            return Vec::new();
        }

        let Some(format_replacement) = build_format_suggestion(&parts) else {
            return Vec::new();
        };

        vec![Arc::new(CodeS44StringConcatFormat::new(
            expression.full_span(),
            processed,
            config.severity(),
            format_replacement,
        ))]
    }
}

fn find_best_concat_chain(expr: &Expression) -> Option<(usize, &Expression)> {
    match expr {
        Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) => {
            let current_parts = collect_concat_parts_count(expr);
            let lhs_best = find_best_concat_chain(lhs);
            let rhs_best = find_best_concat_chain(rhs);

            let best_child = lhs_best.iter().chain(rhs_best.iter()).max_by_key(|(count, _)| *count);
            match best_child {
                Some((child_count, child_expr)) if *child_count > current_parts => {
                    Some((*child_count, child_expr))
                }
                _ if current_parts >= 3 && contains_string_literal(expr) => {
                    Some((current_parts, expr))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn collect_concat_parts_count(expr: &Expression) -> usize {
    let mut parts = Vec::new();
    collect_concat_parts(expr, &mut parts);
    parts.len()
}

fn contains_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::String(_, _, _) => true,
        Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) => {
            contains_string_literal(lhs) || contains_string_literal(rhs)
        }
        _ => false,
    }
}

fn collect_concat_parts(expr: &Expression, parts: &mut Vec<String>) {
    match expr {
        Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) => {
            collect_concat_parts(lhs, parts);
            collect_concat_parts(rhs, parts);
        }
        Expression::String(value, _, _) => parts.push(format!("\"{value}\"")),
        Expression::Variable(name, _) => parts.push(name.clone()),
        Expression::UnaryCommand(_, _, _) => {
            parts.push(expr.source(false));
        }
        _ => parts.push(expr.source(false)),
    }
}

fn build_format_suggestion(parts: &[String]) -> Option<String> {
    let mut format_parts = Vec::new();
    let mut placeholders = Vec::new();

    for part in parts {
        if let Some(stripped) = part.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            format_parts.push(stripped.replace('%', "%%"));
        } else {
            placeholders.push(part.clone());
            format_parts.push(format!("%{}", placeholders.len()));
        }
    }

    if placeholders.is_empty() {
        return None;
    }

    let format_string = format_parts.join("");
    let args = placeholders.join(", ");
    Some(format!("format [\"{format_string}\", {args}]") )
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS44StringConcatFormat {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    suggestion: String,
}

impl Code for CodeS44StringConcatFormat {
    fn ident(&self) -> &'static str {
        "L-S44"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#string_concat_format")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Use `format` instead of string concatenation".to_string()
    }

    fn label_message(&self) -> String {
        "chained concatenation".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(self.suggestion.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS44StringConcatFormat {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, suggestion: String) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            suggestion,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
