use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, BinaryCommand, Expression};

crate::analyze::lint!(LintS39ReplacedFunctions);

impl Lint<LintData> for LintS39ReplacedFunctions {
    fn ident(&self) -> &'static str {
        "replaced_functions"
    }

    fn sort(&self) -> u32 {
        390
    }

    fn description(&self) -> &'static str {
        "Checks for usage of BIS functions that have been replaced by commands. The lint will skip any files starting with `_bi_` or `bis` as these are considered modified internal Arma files."
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
[a,b,c] call BIS_fnc_selectRandom;
```
**Correct**
```sqf
selectRandom [a,b,c];
```

### Explanation

Some BIS functions have been replaced by commands, which are more efficient and easier to read. Using the native commands is recommended for better performance.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

pub enum ReplacedShaped {
    Unary,
    Binary,
}

const REPLACED_FUNCTIONS: &[(&str, &str, ReplacedShaped)] = &[
    ("BIS_fnc_selectRandom", "selectRandom", ReplacedShaped::Unary),
    ("BIS_fnc_selectRandomWeighted", "selectRandomWeighted", ReplacedShaped::Unary),
    ("BIS_fnc_areEqual", "isEqualTo", ReplacedShaped::Binary),
    ("BIS_fnc_vectorMultiply", "vectorMultiply", ReplacedShaped::Binary),
    ("BIS_fnc_vectorDivide", "vectorDivide", ReplacedShaped::Binary),
    ("BIS_fnc_vectorAdd", "vectorAdd", ReplacedShaped::Binary),
    ("BIS_fnc_vectorSubtract", "vectorSubtract", ReplacedShaped::Binary),
    ("BIS_fnc_vectorDiff", "vectorDiff", ReplacedShaped::Binary),
    ("BIS_fnc_linearConversion", "linearConversion", ReplacedShaped::Unary),
    ("BIS_fnc_param", "param", ReplacedShaped::Unary),
    ("BIS_fnc_MP", "remoteExec", ReplacedShaped::Unary),
];

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

        let Some(mapping) = processed.mapping(target.span().start) else {
            return Vec::new();
        };
        let Some((source, _)) = processed.source(mapping.source()) else {
            return Vec::new();
        };
        if source.as_str().contains("_bi_") || source.as_str().contains("bis") {
            return Vec::new();
        }

        let Expression::BinaryCommand(BinaryCommand::Named(cmd), lhs, rhs, _) = target else {
            return Vec::new();
        };
        
        if !cmd.eq_ignore_ascii_case("call") {
            return Vec::new();
        }

        let Expression::Variable(function_name, _) = &**rhs else {
            return Vec::new();
        };

        let Some((_, replacement, shape)) = REPLACED_FUNCTIONS.iter().find(|(name, _, _)| name.eq_ignore_ascii_case(function_name)) else {
            return Vec::new();
        };

        let replacement_cmd = match shape {
            ReplacedShaped::Unary => Expression::UnaryCommand(
                crate::UnaryCommand::Named(replacement.to_string()),
                lhs.clone(),
                lhs.span(),
            ),
            ReplacedShaped::Binary => {
                let Expression::Array(parts, _) = &**lhs else {
                    return Vec::new();
                };
                if parts.len() != 2 {
                    return Vec::new();
                }
                let lhs = Box::new(parts[0].clone());
                let lhs_span = lhs.span();
                let rhs = Box::new(parts[1].clone());
                Expression::BinaryCommand(
                    crate::BinaryCommand::Named(replacement.to_string()),
                    lhs,
                    rhs,
                    lhs_span,
                )
            },
        };

        vec![Arc::new(CodeS39ReplacedFunctions::new(
            target.full_span(),
            processed,
            config.severity(),
            replacement_cmd.source(false),
            replacement.to_string(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS39ReplacedFunctions {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    replacement: String,
    replacement_cmd: String,
}

impl Code for CodeS39ReplacedFunctions {
    fn ident(&self) -> &'static str {
        "L-S39"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#replaced_functions")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!(
            "Function has been replaced by command: `{}`",
            self.replacement_cmd
        )
    }

    fn label_message(&self) -> String {
        format!("use `{}`", self.replacement_cmd)
    }

    fn suggestion(&self) -> Option<String> {
        if self.replacement_cmd == "remoteExec" || self.replacement_cmd == "param" {
            None
        } else {
            Some(self.replacement.clone())
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS39ReplacedFunctions {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, replacement: String, replacement_cmd: String) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            replacement,
            replacement_cmd,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
