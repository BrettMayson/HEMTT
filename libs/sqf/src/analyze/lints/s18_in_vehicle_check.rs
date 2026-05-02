use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, NularCommand, UnaryCommand};

crate::analyze::lint!(LintS18InVehicleCheck);

impl Lint<LintData> for LintS18InVehicleCheck {
    fn ident(&self) -> &'static str {
        "in_vehicle_check"
    }

    fn sort(&self) -> u32 {
        180
    }

    fn description(&self) -> &'static str {
        "Recommends using `isNull objectParent X` instead of `vehicle X == X`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
if (vehicle player == player) then { ... };
```
**Correct**
```sqf
if (isNull objectParent player) then { ... };
```

### Explanation

Using `isNull objectParent x` is faster and more reliable than `vehicle x == x` for checking if a unit is currently in a vehicle.
"
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
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Eq | BinaryCommand::NotEq, lhs, rhs, _) = target else {
            return Vec::new();
        };
        
        let Some((check, check_span)) = is_in_vehicle_check(lhs, rhs).or_else(|| is_in_vehicle_check(rhs, lhs)) else {
            return Vec::new();
        };

        let original = crate::analyze::recover_original_source(processed, check_span.start);
        vec![Arc::new(CodeS18InVehicleCheck::new(
            target.full_span(),
            processed,
            config.severity(),
            check,
            matches!(target, Expression::BinaryCommand(BinaryCommand::NotEq, _, _, _)),
            original,
        ))]
    }
}

fn is_in_vehicle_check(lhs: &Expression, rhs: &Expression) -> Option<(String, Range<usize>)> {
    // vehicle x == x
    let Expression::UnaryCommand(UnaryCommand::Named(name), object, _) = lhs else {
        return None;
    };
    if !name.eq_ignore_ascii_case("vehicle") {
        return None;
    }
    let var_and_span = match &**object {
        Expression::Variable(var, span) | Expression::NularCommand(NularCommand { name: var}, span) => Some((var.clone(), span.clone())),
        _ => None,
    }?;
    let (Expression::Variable(var2, _) | Expression::NularCommand(NularCommand { name: var2}, _)) = rhs else {
        return None;
    };
    if var_and_span.0 == *var2 {
        return Some((var_and_span.0, var_and_span.1));
    }
    None
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS18InVehicleCheck {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    ident: String,
    negated: bool,
    original_source: Option<String>,
}

impl Code for CodeS18InVehicleCheck {
    fn ident(&self) -> &'static str {
        "L-S18"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#in_vehicle_check")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Using `vehicle` to check if a unit is in a vehicle is inefficient".to_string()
    }

    fn label_message(&self) -> String {
        if self.negated {
            "inefficient \"in vehicle\" check".to_string()
        } else {
            "inefficient \"not in vehicle\" check".to_string()
        }
    }

    fn suggestion(&self) -> Option<String> {
        let var = self.original_source.as_ref().unwrap_or(&self.ident);
        Some(
            format!("{} objectParent {}", if self.negated { "!isNull" } else { "isNull" }, var),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS18InVehicleCheck {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, ident: String, negated: bool, original_source: Option<String>) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            ident,
            negated,
            original_source,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
