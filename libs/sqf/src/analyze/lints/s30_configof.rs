use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression, NularCommand, UnaryCommand};

crate::analyze::lint!(LintS30ConfigOf);

impl Lint<LintData> for LintS30ConfigOf {
    fn ident(&self) -> &'static str {
        "config_of"
    }

    fn sort(&self) -> u32 {
        300
    }

    fn description(&self) -> &'static str {
        "Checks for typeOf used with configFile when configOf could be used instead"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
private _name = getText(configFile >> 'CfgVehicles' >> typeOf _vehicle >> 'displayName');
```
**Correct**
```sqf
private _name = getText(configOf _vehicle >> 'displayName');
```

### Explanation

The `configOf` command is specifically designed to retrieve configuration data for a given object, and is faster and more efficient than using `typeOf` in conjunction with `configFile`.
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
        let Expression::BinaryCommand(BinaryCommand::ConfigPath, lhs, rhs, _) = target else {
            return Vec::new();
        };

        if let Expression::UnaryCommand(UnaryCommand::Named(rhs_cmd), rhs_rhs, _ ) = &**rhs
        && rhs_cmd.eq_ignore_ascii_case("typeof")
        && let Expression::BinaryCommand(BinaryCommand::ConfigPath, lhs_lhs, lhs_rhs, _) = &**lhs
        && let Expression::NularCommand(NularCommand { name }, _) = lhs_lhs.as_ref()
        && name.eq_ignore_ascii_case("configfile")
        && let Expression::String(str, _, _) = lhs_rhs.as_ref()
        && (str.eq_ignore_ascii_case("cfgvehicles") || str.eq_ignore_ascii_case("cfgammo"))
    {
        return vec![Arc::new(CodeS30ConfigOf::new(rhs_rhs.source(), lhs_lhs.span().start .. rhs_rhs.span().end, processed, config.severity()))];
    }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS30ConfigOf {
    variable: String,
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS30ConfigOf {
    fn ident(&self) -> &'static str {
        "L-S30"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#config_of")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Use configOf instead of typeOf with configFile for variable `{}`", self.variable)
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("configOf {}", self.variable))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS30ConfigOf {
    #[must_use]
    pub fn new(variable: String, span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
            variable,
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
