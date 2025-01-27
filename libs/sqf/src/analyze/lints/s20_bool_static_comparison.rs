use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Codes, Diagnostic, Processed, Severity}};

use crate::{analyze::LintData, BinaryCommand, Expression};

crate::analyze::lint!(LintS20BoolStaticComparison);

impl Lint<LintData> for LintS20BoolStaticComparison {
    fn ident(&self) -> &'static str {
        "bool_static_comparison"
    }

    fn sort(&self) -> u32 {
        200
    }

    fn description(&self) -> &'static str {
        "Checks for a variable being compared to `true` or `false`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
if (_x == true) then {};
if (_y == false) then {};
```
**Correct**
```sqf
if (_x) then {};
if (!_y) then {};
```
"
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
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Eq | BinaryCommand::NotEq, lhs, rhs, _) = target else {
            return Vec::new();
        };
        
        let Some((ident, against_true)) = is_static_comparison(lhs, rhs).or_else(|| is_static_comparison(rhs, lhs)) else {
            return Vec::new();
        };

        vec![Arc::new(CodeS20BoolStaticComparison::new(
            target.full_span(),
            processed,
            config.severity(),
            ident,
            against_true,
            matches!(target, Expression::BinaryCommand(BinaryCommand::NotEq, _, _, _)),
        ))]
    }
}

fn is_static_comparison(lhs: &Expression, rhs: &Expression) -> Option<(String, bool)> {
    match rhs {
        Expression::Boolean(against_true, _) => {
            match lhs {
                Expression::Variable(var, _) => Some((var.clone(), *against_true)),
                _ => None,
            }
        }
        _ => None,
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS20BoolStaticComparison {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    ident: String,
    against_true: bool,
    negated: bool,
}

impl Code for CodeS20BoolStaticComparison {
    fn ident(&self) -> &'static str {
        "L-S20"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#bool_static_comparison")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Variable compared to static boolean".to_string()
    }

    fn label_message(&self) -> String {
        "compared to static boolean".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(if self.against_true {
            if self.negated {
                format!("!{}", self.ident)
            } else {
                self.ident.clone()
            }
        } else if self.negated {
            self.ident.clone()
        } else {
            format!("!{}", self.ident)
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS20BoolStaticComparison {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, ident: String, against_true: bool, negated: bool) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            ident,
            against_true,
            negated,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
