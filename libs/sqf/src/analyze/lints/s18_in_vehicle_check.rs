use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::SqfLintData, BinaryCommand, Expression, NularCommand, UnaryCommand};

crate::lint!(LintS18InVehicleCheck);

impl Lint<SqfLintData> for LintS18InVehicleCheck {
    fn ident(&self) -> &str {
        "in_vehicle_check"
    }

    fn sort(&self) -> u32 {
        180
    }

    fn description(&self) -> &str {
        "Recommends using `isNull objectParent X` instead of `vehicle X == X`"
    }

    fn documentation(&self) -> &str {
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
        LintConfig::help()
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
        let Expression::BinaryCommand(BinaryCommand::Eq | BinaryCommand::NotEq, lhs, rhs, _) = target else {
            return Vec::new();
        };
        
        let Some(check) = is_in_vehicle_check(lhs, rhs).or_else(|| is_in_vehicle_check(rhs, lhs)) else {
            return Vec::new();
        };

        vec![Arc::new(CodeS18InVehicleCheck::new(
            target.full_span(),
            processed,
            config.severity(),
            check,
            matches!(target, Expression::BinaryCommand(BinaryCommand::NotEq, _, _, _)),
        ))]
    }
}

fn is_in_vehicle_check(lhs: &Expression, rhs: &Expression) -> Option<String> {
    // vehicle x == x
    let Expression::UnaryCommand(UnaryCommand::Named(name), object, _) = lhs else {
        return None;
    };
    if name.to_lowercase() != "vehicle" {
        return None;
    }
    let (Expression::Variable(var, _) | Expression::NularCommand(NularCommand { name: var}, _)) = &**object else {
        return None;
    };
    let (Expression::Variable(var2, _) | Expression::NularCommand(NularCommand { name: var2}, _)) = rhs else {
        return None;
    };
    if var == var2 {
        return Some(var.clone());
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
}

impl Code for CodeS18InVehicleCheck {
    fn ident(&self) -> &'static str {
        "L-S18"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#in_vehicle_check")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Using `vehicle` to check if a unit is in a vehicle is innefficient".to_string()
    }

    fn label_message(&self) -> String {
        if self.negated {
            "Innefficient \"in vehicle\" check".to_string()
        } else {
            "Innefficient \"not in vehicle\" check".to_string()
        }
    }

    fn suggestion(&self) -> Option<String> {
        Some(
            format!("{} objectParent {}", if self.negated { "!isNull" } else { "isNull" }, self.ident),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS18InVehicleCheck {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, var: String, negated: bool) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            ident: var,
            negated,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
