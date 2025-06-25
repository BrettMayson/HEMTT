use std::{ops::Range, sync::Arc};

use float_ord::FloatOrd;
use hemtt_common::config::LintConfig;
use hemtt_workspace::{lint::{AnyLintRunner, Lint, LintRunner}, reporting::{Code, Diagnostic, Processed, Severity}};

use crate::{analyze::LintData, BinaryCommand, Expression, UnaryCommand};

crate::analyze::lint!(LintS06FindInStr);

impl Lint<LintData> for LintS06FindInStr {
    fn ident(&self) -> &'static str {
        "find_in_str"
    }

    fn sort(&self) -> u32 {
        60
    }

    fn description(&self) -> &'static str {
        "Checks for `find` commands that can be replaced with `in`"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```sqf
if (_haystack find _needle > -1) ...
```
**Correct**
```sqf
if (_needle in _haystack) ...
```

### Explanation

The `in` command is faster than `find` when searching for a substring in a string."
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
    type Target = Expression;
    
    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        config: &LintConfig,
        processed: Option<&hemtt_workspace::reporting::Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Self::Target,
        _data: &LintData,
    ) -> hemtt_workspace::reporting::Codes {
        let Some(processed) = processed else {
            return Vec::new();
        };
        let Expression::BinaryCommand(BinaryCommand::Greater, search, compare, _) = target else {
            return Vec::new();
        };
        let Expression::UnaryCommand(UnaryCommand::Minus, to, _) = &**compare else {
            return Vec::new();
        };
        let Expression::Number(FloatOrd(num), _) = &**to else {
            return Vec::new();
        };
        if (num - 1.0).abs() > f32::EPSILON {
            return Vec::new();
        }
        let Expression::BinaryCommand(BinaryCommand::Named(name), haystack, needle, _) = &**search
        else {
            return Vec::new();
        };
        if name.to_lowercase() != "find" {
            return Vec::new();
        }
    
        if let Expression::String(needle_str, _, _) = &**needle {
            let haystack_str = match &**haystack {
                Expression::String(s, _, wrapper) => {
                    format!("{}{s}{}", wrapper.as_str(), wrapper.as_str())
                }
                Expression::Variable(name, _) => name.to_string(),
                _ => return Vec::new(),
            };
            return vec![Arc::new(CodeS06FindInStr::new(
                haystack.span().start..target.full_span().end,
                (haystack_str, haystack.span()),
                (format!("\"{needle_str}\""), needle.span()),
                processed,
                config.severity(),
            ))];
        }
        Vec::new()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS06FindInStr {
    span: Range<usize>,
    haystack: (String, Range<usize>),
    needle: (String, Range<usize>),

    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS06FindInStr {
    fn ident(&self) -> &'static str {
        "L-S06"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/sqf.html#find_in_str")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("string search using `in` is faster than `find`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!(
            "{} in {}",
            self.needle.0.as_str(),
            self.haystack.0.as_str()
        ))
    }

    fn label_message(&self) -> String {
        String::from("using `find` with -1")
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS06FindInStr {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        haystack: (String, Range<usize>),
        needle: (String, Range<usize>),
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            haystack,
            needle,

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
