use std::{collections::HashMap, ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity}, WorkspacePath,
};

use crate::{analyze::SqfLintData, BinaryCommand, Expression, Statement};

crate::analyze::lint!(LintS24MarkerSpam);

impl Lint<SqfLintData> for LintS24MarkerSpam {
    fn ident(&self) -> &str {
        "marker_update_spam"
    }

    fn sort(&self) -> u32 {
        240
    }

    fn description(&self) -> &str {
        "Checks for repeated calls to global marker updates"
    }

    fn documentation(&self) -> &str {
        r#"### Example

**Incorrect**
```sqf
"my_marker" setMarkerAlpha 0.5;
"my_marker" setMarkerDir 90;
"my_marker" setMarkerSize [100, 200];
"my_marker" setMarkerShape "RECTANGLE";
```
**Correct**
```sqf
"my_marker" setMarkerAlphaLocal 0.5;
"my_marker" setMarkerDirLocal 90;
"my_marker" setMarkerSizeLocal [100, 200];
"my_marker" setMarkerShape "RECTANGLE";
```

### Explanation

The `setMarker*` commands send the entire state of the marker to all clients. This can be very expensive if done repeatedly.  
Using the `setMarker*Local` on all calls except the last one will reduce the amount of data sent over the network.
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::help()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = crate::Statements;

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
        
        let mut codes: Codes = Vec::new();

        let mut pending: HashMap<String, Vec<(String, Range<usize>)>> = HashMap::new();

        for statement in target.content() {
            match statement {
                Statement::Expression(Expression::BinaryCommand(BinaryCommand::Named(cmd), name, _, cmd_span), _) if is_marker_global(cmd) => {
                    let Some(name) = marker_name(name) else {
                        continue;
                    };
                    pending.entry(name.clone()).or_default().push((cmd.to_string(), cmd_span.clone()));
                }
                Statement::Expression(_, _) => {}
                Statement::AssignGlobal(name, _, _) | Statement::AssignLocal(name, _, _) => {
                    if let Some(existing) = pending.remove(name) {
                        if existing.len() > 1 {
                            codes.push(Arc::new(CodeS24MarkerSpam::new(existing, processed, config.severity())));
                        }
                    }
                }
            }
        }

        for (_, calls) in pending {
            if calls.len() > 1 {
                codes.push(Arc::new(CodeS24MarkerSpam::new(calls, processed, config.severity())));
            }
        }

        codes
    }
}

fn is_marker_global(cmd: &str) -> bool {
    cmd.starts_with("setMarker") && !cmd.ends_with("Local")
}

fn marker_name(var: &Expression) -> Option<String> {
    match var {
        Expression::Variable(name, _) => Some(name.clone()),
        Expression::String(name, _, _) => Some(name.to_string()),
        _ => None,
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS24MarkerSpam {
    calls: Vec<(String, Range<usize>)>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS24MarkerSpam {
    fn ident(&self) -> &'static str {
        "L-S24"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#marker_update_spam")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Repeated calls to global marker updates".to_string()
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn note(&self) -> Option<String> {
        Some("Global marker commands update the entire state of the marker".to_string())
    }

    fn help(&self) -> Option<String> {
        Some("Using `setMarker*Local` on all except the last call reduces network updates".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS24MarkerSpam {
    #[must_use]
    pub fn new(calls: Vec<(String, Range<usize>)>, processed: &Processed, severity: Severity) -> Self {
        Self {
            calls,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(mut diag) = Diagnostic::from_code_processed(&self, self.calls.first().expect("at least one call").1.clone(), processed) else {
            return self;
        };
        diag = diag.clear_labels();
        let last = self.calls.last().expect("at least one call");
        let Some(info) = get_span_info(&last.1, processed) else {
            return self;
        };
        diag = diag.with_label(Label::secondary(
            info.0,
            info.1,
        ).with_message("Last marker update, should remain global"));
        for (cmd, span) in self.calls.iter().rev().skip(1) {
            let Some(info) = get_span_info(span, processed) else {
                continue;
            };
            diag = diag.with_label(Label::primary(
                info.0,
                info.1,
            ).with_message(format!("Use {cmd}Local")));
        }
        self.diagnostic = Some(diag);
        self
    }
}

fn get_span_info(span: &Range<usize>, processed: &Processed) -> Option<(WorkspacePath, Range<usize>)> {
    let map_start = processed.mapping(span.start)?;
    let map_end = processed.mapping(span.end)?;
    let map_file = processed.source(map_start.source())?;
    Some((
        map_file.0.clone(),
        map_start.original_start()..map_end.original_start(),
    ))
}
