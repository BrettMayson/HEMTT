use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, LintEnabled, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Item, Number, Value};

crate::analyze::lint!(LintC12MathCouldBeUnquoted);

impl Lint<LintData> for LintC12MathCouldBeUnquoted {
    fn ident(&self) -> &'static str {
        "math_could_be_unquoted"
    }

    fn sort(&self) -> u32 {
        120
    }

    fn description(&self) -> &'static str {
        "Reports on quoted math statements that could be evaulated at build-time"
    }

    fn documentation(&self) -> &'static str {
        "### Example

**Incorrect**
```hpp
x = '1+1';
```

**Correct**
```hpp
x = 1+1; // hemtt will evaluate at build-time to 2
```

### Explanation
Quoted math statements will have to be evaulated in-game
"
    }

    fn default_config(&self) -> LintConfig {
        // false-positives are possible
        LintConfig::help().with_enabled(LintEnabled::Pedantic)
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Value;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &crate::Value,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let mut codes = Vec::new();
        let Some(processed) = processed else {
            return vec![];
        };
        match target {
            Value::Array(arr) => {
                for item in &arr.items {
                    let Item::Str(str) = item else { continue };
                    if let Some(code) = check(str, processed, config) {
                        codes.push(code);
                    }
                }
            }
            Value::Str(str) => {
                if let Some(code) = check(str, processed, config) {
                    codes.push(code);
                }
            }
            _ => {}
        }

        codes
    }
}

fn check(
    target_str: &crate::Str,
    processed: &Processed,
    config: &LintConfig,
) -> Option<Arc<dyn Code>> {
    let raw_string = target_str.value();
    // check if it contains some kind of math ops (avoid false positives from `displayName = "556";`)
    if !(raw_string.contains('+')
        || raw_string.contains('-')
        || raw_string.contains('*')
        || raw_string.contains('/'))
    {
        return None;
    }
    // attempt to parse it as a number
    let num = Number::try_evaulation(raw_string, target_str.span())?;
    let span = target_str.span().start + 1..target_str.span().end - 1;
    Some(Arc::new(Code12MathCouldBeUnquoted::new(
        span,
        processed,
        format!("reducible to: {num}"),
        config.severity(),
    )))
}

#[allow(clippy::module_name_repetitions)]
pub struct Code12MathCouldBeUnquoted {
    span: Range<usize>,
    label: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code12MathCouldBeUnquoted {
    fn ident(&self) -> &'static str {
        "L-C12"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#math_could_be_unquoted")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Math could be unquoted".to_string()
    }

    fn label_message(&self) -> String {
        self.label.clone()
    }

    fn note(&self) -> Option<String> {
        Some("Could remove quotes to allow evaluation at build-time".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code12MathCouldBeUnquoted {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        processed: &Processed,
        label: String,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            label,
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
