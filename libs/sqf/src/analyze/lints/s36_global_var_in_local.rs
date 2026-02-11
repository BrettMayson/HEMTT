use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS35CountSkippable);

impl Lint<LintData> for LintS35CountSkippable {
    fn ident(&self) -> &'static str {
        "global_var_in_local"
    }
    fn sort(&self) -> u32 {
        360
    }
    fn description(&self) -> &'static str {
        "Checks for use of global variables in `private` and `param` variable declarations"
    }
    fn documentation(&self) -> &'static str {
        r#"
        ### Example

**Incorrect**
```sqf
private ["z"];
```
**Correct**
```sqf
private ["_z"];
```
"#
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
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
        fn has_global_var(exp: &Expression) -> Option<&Range<usize>> {
            match exp {
                // check var name for private (`private "var"`)
                Expression::String(var, span, _) => if !(var.is_empty() || var.starts_with('_')) { return Some(span); },
                Expression::Array(outer_arr, _) => {
                    for e in outer_arr {
                        match e {
                            // check any element for global (`params ["var"]` or `private ["var"]`)
                            Expression::String(var, span, _) => {
                                if !(var.is_empty() || var.starts_with('_')) { return Some(span); }
                            }
                            Expression::Array(innest_arr, _) => {
                                // check any first element in inner array (`params [["var"]]`)
                                let Some(Expression::String(var, span, _)) = innest_arr.first() else { continue; };
                                if !(var.is_empty() || var.starts_with('_')) { return Some(span); }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            None
        }
        let Some(processed) = processed else {
            return Vec::new();
        };
        match target {
            Expression::BinaryCommand(BinaryCommand::Named(cname), _, rhs, _)
            | Expression::UnaryCommand(UnaryCommand::Named(cname), rhs, _) => {
                if (cname.eq_ignore_ascii_case("private") || cname.eq_ignore_ascii_case("params")) && let Some(span) = has_global_var(rhs) {
                    return vec![Arc::new(Code36GlobalVarInLocal::new(
                        cname.clone(),
                        span.clone(),
                        processed,
                        config.severity(),
                    ))];
                }
            }
            _ => {}
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Code36GlobalVarInLocal {
    cmd: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code36GlobalVarInLocal {
    fn ident(&self) -> &'static str {
        "L-S36"
    }
    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#global_var_in_local")
    }
    fn severity(&self) -> Severity {
        self.severity
    }
    fn message(&self) -> String {
        format!("Global variables cannot be used with {}", self.cmd)
    }
    fn label_message(&self) -> String {
        "global variable".to_string()
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code36GlobalVarInLocal {
    #[must_use]
    pub fn new(cmd: String, span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            cmd,
            span,
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
