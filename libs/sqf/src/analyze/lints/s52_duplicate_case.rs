use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity},
    WorkspacePath,
};

use crate::{analyze::LintData, BinaryCommand, Expression, Statement, UnaryCommand};

crate::analyze::lint!(LintS52DuplicateCase);

impl Lint<LintData> for LintS52DuplicateCase {
    fn ident(&self) -> &'static str {
        "duplicate_case"
    }

    fn sort(&self) -> u32 {
        520
    }

    fn description(&self) -> &'static str {
        "Checks for duplicate cases in switch statements"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
switch (_value) do {
    case 1: { "one" };
    case 2: { "two" };
    case 1: { "one again" };
};
```

**Correct**
```sqf
switch (_value) do {
    case 1: { "one" };
    case 2: { "two" };
    case 3: { "three" };
};
```

### Explanation

Having duplicate case labels in a switch statement is likely a mistake. Only the first case will be executed, and the duplicate case will never be reached.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Expression;

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

        // Look for switch (expr) do { ... }
        let Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _) = target else {
            return Vec::new();
        };

        if !cmd.eq_ignore_ascii_case("do") {
            return Vec::new();
        }

        // Check if this is a switch statement
        let Expression::UnaryCommand(UnaryCommand::Named(unary), _, _) = lhs.as_ref() else {
            return Vec::new();
        };

        if !unary.eq_ignore_ascii_case("switch") {
            return Vec::new();
        }

        // The right side should be a Code block containing case statements
        let Expression::Code(body) = rhs.as_ref() else {
            return Vec::new();
        };

        // Check if this is a switch statement by looking for case expressions
        let mut case_values: Vec<(String, Range<usize>)> = Vec::new();
        let mut codes: Codes = Vec::new();

        for body_statement in body.content() {
            let Statement::Expression(case_expr, _) = body_statement else {
                continue;
            };
            
            let case = if let Expression::BinaryCommand(BinaryCommand::Associate, left, _right, _) = case_expr {
                left
            } else {
                case_expr
            };
            if let Expression::UnaryCommand(UnaryCommand::Named(name), value_expr, _) = case
                && name.eq_ignore_ascii_case("case") {
                    let case_source = value_expr.source(false);
                    let case_span = value_expr.span();

                    for (existing_value, existing_span) in &case_values {
                        if existing_value == &case_source {
                            codes.push(Arc::new(CodeS52DuplicateCase::new(
                                case_span.clone(),
                                case_source.clone(),
                                existing_span.clone(),
                                processed,
                                config.severity(),
                            )));
                        }
                    }

                    case_values.push((case_source, case_span));
                }
        }

        codes
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS52DuplicateCase {
    span: Range<usize>,
    value: String,
    first_span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS52DuplicateCase {
    fn ident(&self) -> &'static str {
        "L-S52"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#duplicate_case")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Duplicate case `{}` in switch statement", self.value)
    }

    fn label_message(&self) -> String {
        "duplicate case".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS52DuplicateCase {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        value: String,
        first_span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            value,
            first_span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.span.clone(), processed)
        else {
            return self;
        };

        // Try to get info about the first span
        if let Some((path, span)) = get_span_info(self.first_span.clone(), processed) {
            diag = diag.with_label(
                Label::secondary(path, span)
                    .with_message("first case here"),
            );
        }
        self.diagnostic = Some(diag);
        self
    }
}

fn get_span_info(span: Range<usize>, processed: &Processed) -> Option<(WorkspacePath, Range<usize>)> {
    let map_start = processed.mapping(span.start)?;
    let map_end = processed.mapping(span.end)?;
    let map_file = processed.source(map_start.source())?;
    Some((
        map_file.0.clone(),
        map_start.original_start()..map_end.original_end(),
    ))
}
