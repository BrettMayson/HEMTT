use std::{cmp::Ordering, ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::SqfLintData, Expression, UnaryCommand};

crate::analyze::lint!(LintS08FormatArgs);

impl Lint<SqfLintData> for LintS08FormatArgs {
    fn ident(&self) -> &'static str {
        "format_args"
    }

    fn sort(&self) -> u32 {
        80
    }

    fn description(&self) -> &'static str {
        "Checks for format commands with incorrect argument counts"
    }

    fn documentation(&self) -> &'static str {
r#"### Example

**Incorrect**
```sqf
private _text = format ["%1", "Hello", "World"];
```
**Correct**
```sqf
private _text = format ["%1", "Hello World"];
```
**Incorrect**
```sqf
private _text = format ["%1 %2", "Hello World"];
```
**Correct**
```sqf
private _text = format ["%1 %2", "Hello", "World"];
```

### Explanation

The `format` and `formatText` commands requires the correct number of arguments to match the format string."#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn minimum_severity(&self) -> Severity {
        Severity::Warning
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = crate::Expression;
    
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &SqfLintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Named(name), expression, _) = target else {
            return Vec::new();
        };
        if name.to_lowercase() != "format" && name.to_lowercase() != "formattext" {
            return Vec::new();
        }
        let Expression::Array(args, _) = &**expression else {
            return Vec::new();
        };
        if args.is_empty() {
            return vec![Arc::new(CodeS08FormatArgs::new(
                target.full_span(),
                "format string: empty array".to_string(),
                processed,
                config.severity(),
            ))];
        }
        let Expression::String(format, _, _) = &args[0] else {
            return Vec::new();
        };
    
        #[allow(clippy::option_if_let_else)]
        if let Some(problem) = get_format_problem(format, args.len() - 1) {
            vec![Arc::new(CodeS08FormatArgs::new(
                target.full_span(),
                problem,
                processed,
                config.severity(),
            ))]
        } else {
            Vec::new()
        }
    }
}

#[must_use]
fn get_format_problem(input: &str, extra_args: usize) -> Option<String> {
    let format = format!("{input} ",); // add extra terminator

    let mut tokens: Vec<usize> = Vec::new();
    let mut token_active = false;
    let mut token_start = 0;
    for (i, c) in format.chars().enumerate() {
        let outside_token = !token_active || i > token_start;
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
        if !token_active && c == '%' && outside_token {
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

#[allow(clippy::module_name_repetitions)]
pub struct CodeS08FormatArgs {
    span: Range<usize>,
    problem: String,

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS08FormatArgs {
    fn ident(&self) -> &'static str {
        "L-S08"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#format_args")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        self.problem.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS08FormatArgs {
    #[must_use]
    pub fn new(span: Range<usize>, problem: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            problem,

            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
