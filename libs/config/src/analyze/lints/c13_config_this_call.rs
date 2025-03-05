use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, LintEnabled, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Value};

crate::analyze::lint!(LintC13ConfigThisCall);

impl Lint<LintData> for LintC13ConfigThisCall {
    fn ident(&self) -> &'static str {
        "config_this_call"
    }

    fn sort(&self) -> u32 {
        130
    }

    fn description(&self) -> &'static str {
        "Checks for usage of `_this call`, where `_this` is not necessary - in config text"
    }

    fn documentation(&self) -> &'static str {
        "### Example

**Incorrect**
```hpp
statement = \"_this call _my_function\";
```

**Correct**
```hpp
statement = \"call _my_function\";
```

### Explanation

When using `call`, the called code will inherit `_this` from the calling scope. This means that `_this` is not necessary in the call, and can be omitted for better performance.
"
    }

    fn default_config(&self) -> LintConfig {
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
        let Some(processed) = processed else {
            return vec![];
        };
        let Value::Str(target_str) = target else {
            return vec![];
        };

        let raw_string = target_str.value();
        let count_this_call: Vec<_> = raw_string.match_indices("_this call ").collect();
        if count_this_call.is_empty() {
            return vec![];
        }
        let count_this: Vec<_> = raw_string.match_indices("_this").collect();
        if count_this_call.len() != count_this.len() {
            // must be some other use of _this
            return vec![];
        }

        println!(
            "{}-{} from {}",
            count_this_call.len(),
            count_this.len(),
            raw_string
        );

        let span = target_str.span().start + 1..target_str.span().end - 1;

        vec![Arc::new(Code13ConfigThisCall::new(
            span,
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Code13ConfigThisCall {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code13ConfigThisCall {
    fn ident(&self) -> &'static str {
        "L-C13"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#config_this_call")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Unnecessary `_this` in `call`".to_string()
    }

    fn label_message(&self) -> String {
        String::new()
    }

    fn note(&self) -> Option<String> {
        Some("`call` inherits `_this` from the calling scope".to_string())
    }
    fn help(&self) -> Option<String> {
        Some("Remove `_this` from the call".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code13ConfigThisCall {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity) -> Self {
        Self {
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
