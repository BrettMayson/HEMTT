use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, UnaryCommand, analyze::LintData};

crate::analyze::lint!(LintS46IsEqualType);

impl Lint<LintData> for LintS46IsEqualType {
    fn ident(&self) -> &'static str {
        "is_equal_type"
    }

    fn sort(&self) -> u32 {
        460
    }

    fn description(&self) -> &'static str {
        "Checks for comparing two `typeName` results, which can be replaced with `isEqualType`"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```sqf
if (typeName _a == typeName _b) then {};
```
**Correct**
```sqf
if (_a isEqualType _b) then {};
```

### Explanation

`isEqualType` compares the types of two values directly. Comparing `typeName` results instead
builds a string for each side and then compares those strings, which is slower.

See also `static_typename`, which covers `typeName` used on a constant value."
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
    type Target = Expression;

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
        check(target, processed, config)
    }
}

// Pattern: typeName A == typeName B  ->  A isEqualType B
// The negated form maps to !(A isEqualType B).
// Comparing typeName results builds a string per side, isEqualType compares the types directly.

fn check(target: &Expression, processed: &Processed, config: &LintConfig) -> Vec<Arc<dyn Code>> {
    let mut codes = Vec::new();

    let Expression::BinaryCommand(cmd, left, right, _) = target else {
        return codes;
    };
    let negated = match cmd {
        BinaryCommand::Eq => false,
        BinaryCommand::NotEq => true,
        BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isEqualTo") => false,
        BinaryCommand::Named(name) if name.eq_ignore_ascii_case("isNotEqualTo") => true,
        _ => return codes,
    };
    let (Some(lhs), Some(rhs)) = (as_typename(left), as_typename(right)) else {
        return codes;
    };

    codes.push(Arc::new(CodeS46IsEqualType::new(
        target.full_span(),
        lhs.source(true),
        rhs.source(true),
        negated,
        processed,
        config.severity(),
    )) as Arc<dyn Code>);
    codes
}

/// Matches `typeName X`, returning X.
fn as_typename(expr: &Expression) -> Option<&Expression> {
    let Expression::UnaryCommand(UnaryCommand::Named(name), inner, _) = expr else {
        return None;
    };
    if !name.eq_ignore_ascii_case("typeName") {
        return None;
    }
    Some(inner)
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS46IsEqualType {
    span: Range<usize>,
    left: String,
    right: String,
    negated: bool,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS46IsEqualType {
    fn ident(&self) -> &'static str {
        "L-S46"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#is_equal_type")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        String::from("code can be replaced with `isEqualType`")
    }

    fn label_message(&self) -> String {
        String::from("use `isEqualType`")
    }

    fn suggestion(&self) -> Option<String> {
        Some(if self.negated {
            format!("!({} isEqualType {})", self.left, self.right)
        } else {
            format!("{} isEqualType {}", self.left, self.right)
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS46IsEqualType {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        left: String,
        right: String,
        negated: bool,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            left,
            right,
            negated,
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
