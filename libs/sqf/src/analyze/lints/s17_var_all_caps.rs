use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{analyze::SqfLintData, Expression};

crate::lint!(LintS17VarAllCaps);

impl Lint<SqfLintData> for LintS17VarAllCaps {
    fn ident(&self) -> &str {
        "var_all_caps"
    }

    fn sort(&self) -> u32 {
        170
    }

    fn description(&self) -> &str {
        "Checks for global variables that are ALL_CAPS and may actually be a undefined macro"
    }

    fn documentation(&self) -> &str {
        r#"### Configuration

- **ignore**: An array of vars to ignore

```toml
[lints.sqf.var_all_caps]
options.ignore = [
    "XMOD_TEST", "MYMOD_*",
]
```

### Example

**Incorrect**
```sqf
private _z = _y + DO_NOT_EXIST;
```
."#
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
        let Expression::Variable(var, span) = target else {
            return Vec::new();
        };
        if var.starts_with('_') || &var.to_ascii_uppercase() != var || var == "SLX_XEH_COMPILE_NEW"
        {
            return Vec::new();
        }
        if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            if ignore.iter().any(|i| {
                let s = i.as_str().unwrap_or_default();
                if s == var {
                    return true;
                }
                if s.ends_with('*') && var.starts_with(&s[0..s.len() - 1]) {
                    return true;
                }
                false
            }) {
                return Vec::new();
            }
        }
        vec![Arc::new(CodeS17VarAllCaps::new(
            span.clone(),
            var.clone(),
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS17VarAllCaps {
    span: Range<usize>,
    var: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS17VarAllCaps {
    fn ident(&self) -> &'static str {
        "L-S17"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#var_all_caps")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        // print var again here if it's part of a macro
        format!("Var all caps: {}", self.var)
    }

    fn label_message(&self) -> String {
        String::new()
    }
    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS17VarAllCaps {
    #[must_use]
    pub fn new(span: Range<usize>, var: String, processed: &Processed, severity: Severity) -> Self {
        Self {
            span,
            var,
            severity,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
        self
    }
}
