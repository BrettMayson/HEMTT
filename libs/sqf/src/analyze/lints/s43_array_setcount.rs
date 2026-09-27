use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{
    BinaryCommand, Expression, UnaryCommand,
    analyze::LintData,
};

crate::analyze::lint!(LintS43ArraySetcount);

impl Lint<LintData> for LintS43ArraySetcount {
    fn ident(&self) -> &'static str {
        "array_setcount"
    }

    fn sort(&self) -> u32 {
        430
    }

    fn description(&self) -> &'static str {
        "Checks for appending to an array with `_array set [count _array, _value]`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
_a set [count _a, _value];
```

**Correct**
```sqf
_a pushBack _value;
```

### Explanation

Using `set [count _array, value]` to append to an array is less clear and less idiomatic than `pushBack`. The latter communicates the intent directly and is the standard way to append values.
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
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };

        let Expression::BinaryCommand(BinaryCommand::Named(command), lhs, rhs, _) = target else {
            return Vec::new();
        };
        if !command.eq_ignore_ascii_case("set") {
            return Vec::new();
        }

        let Expression::Array(parts, _) = rhs.as_ref() else {
            return Vec::new();
        };
        if parts.len() != 2 {
            return Vec::new();
        }

        let Some(index_expr) = parts.first() else {
            return Vec::new();
        };
        let Some(value_expr) = parts.get(1) else {
            return Vec::new();
        };

        let Expression::UnaryCommand(unary, array, _) = index_expr else {
            return Vec::new();
        };
        if !matches!(unary, UnaryCommand::Named(name) if name.eq_ignore_ascii_case("count")) {
            return Vec::new();
        }

        let array_source = array.source(false);
        let lhs_source = lhs.source(false);
        if array_source != lhs_source {
            return Vec::new();
        }

        vec![Arc::new(CodeS43ArraySetcount::new(
            target.full_span(),
            processed,
            config.severity(),
            lhs_source,
            value_expr.source(false),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS43ArraySetcount {
    span: Range<usize>,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
    array: String,
    value: String,
}

impl Code for CodeS43ArraySetcount {
    fn ident(&self) -> &'static str {
        "L-S43"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#array_setcount")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        "Use `pushBack` instead of `set [count _array, value]`".to_string()
    }

    fn label_message(&self) -> String {
        "array append can use pushBack".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} pushBack {}", self.array, self.value))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS43ArraySetcount {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed, severity: Severity, array: String, value: String) -> Self {
        Self {
            span,
            severity,
            diagnostic: None,
            array,
            value,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
