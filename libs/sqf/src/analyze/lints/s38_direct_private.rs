use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity},
    WorkspacePath,
};

use crate::{
    Expression, Statement, UnaryCommand,
    analyze::LintData,
};

crate::analyze::lint!(LintS38DirectPrivate);

impl Lint<LintData> for LintS38DirectPrivate {
    fn ident(&self) -> &'static str {
        "direct_private"
    }

    fn sort(&self) -> u32 {
        380
    }

    fn description(&self) -> &'static str {
        "Checks for private declarations that are followed by a separate assignment"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
private ["_a"];
_a = 1;
```

**Correct**
```sqf
private _a = 1;
```

### Explanation

Declaring a private variable and assigning it in a separate statement is slower than using a direct initializer. The direct form avoids an extra initialization step.
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

        let mut pending: Vec<(String, Range<usize>)> = Vec::new();
        let mut codes: Codes = Vec::new();

        for statement in target.content() {
            match statement {
                Statement::Expression(Expression::UnaryCommand(UnaryCommand::Named(cname), rhs, _), _) if cname.eq_ignore_ascii_case("private") => {
                    pending.extend(collect_declared_variables(rhs));
                }
                Statement::AssignGlobal(var, _, assignment_span)
                | Statement::AssignLocal(var, _, assignment_span) => {
                    if let Some((declared_var, declaration_span)) = pending.iter().find_map(|(name, span)| (name == var).then_some((name.clone(), span.clone()))) {
                        codes.push(Arc::new(CodeS38DirectPrivate::new(
                            declared_var,
                            declaration_span,
                            assignment_span.clone(),
                            processed,
                            config.severity(),
                        )));
                        pending.retain(|(name, _)| name != var);
                    }
                }
                Statement::Expression(_, _) => {},
            }
        }

        codes
    }
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

#[allow(clippy::module_name_repetitions)]
pub struct CodeS38DirectPrivate {
    variable: String,
    declaration_span: Range<usize>,
    assignment_span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS38DirectPrivate {
    fn ident(&self) -> &'static str {
        "L-S38"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#direct_private")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Use `private {} = <value>` instead of declaring then assigning", self.variable)
    }

    fn label_message(&self) -> String {
        "declaration followed by assignment".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("private {} = <value>", self.variable))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS38DirectPrivate {
    #[must_use]
    pub fn new(
        variable: String,
        declaration_span: Range<usize>,
        assignment_span: Range<usize>,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            variable,
            declaration_span,
            assignment_span,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.assignment_span.clone(), processed) else {
            return self;
        };
        diag = diag.clear_labels();
        if let Some((file, span)) = get_span_info(self.declaration_span.clone(), processed) {
            diag = diag.with_label(Label::secondary(file, span).with_message("declaration"));
        }
        if let Some((file, span)) = get_span_info(self.assignment_span.clone(), processed) {
            diag = diag.with_label(Label::primary(file, span).with_message("assignment"));
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
        map_start.original_start()..map_end.original_start(),
    ))
}
