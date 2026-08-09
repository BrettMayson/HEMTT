use std::{ops::Range, sync::Arc};

use hemtt_common::config::LintConfig;
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed, Severity},
};

use crate::{BinaryCommand, Expression, Statement, analyze::LintData};

crate::analyze::lint!(LintS45ArrayAppend);

impl Lint<LintData> for LintS45ArrayAppend {
    fn ident(&self) -> &'static str {
        "array_append"
    }

    fn sort(&self) -> u32 {
        450
    }

    fn description(&self) -> &'static str {
        "Checks for extending an array with `_array = _array + [...]`"
    }

    fn documentation(&self) -> &'static str {
        r#"### Example

**Incorrect**
```sqf
MY_array = MY_array + ["a", "b"];
```
**Correct**
```sqf
MY_array append ["a", "b"];
```

### Explanation

`array1 + array2` builds a brand new array and the assignment rebinds the variable to it, so the
whole array is copied every time it is extended. `append` adds to the existing array in place.

This changes behaviour when something else still holds a reference to the original array, because
arrays are references and only `+` produces a copy:

```sqf
private _b = MY_array;
MY_array = MY_array + ["a"];  // _b keeps the old array, no "a"
MY_array append ["a"];        // _b sees "a", it is the same array
```

Only reported when the right hand side is an array literal, so numeric and string `+` are not
affected."#
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
    type Target = crate::Statement;

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
        let (assigned, expression, range) = match target {
            Statement::AssignGlobal(name, expression, range)
            | Statement::AssignLocal(name, expression, range) => (name, expression, range),
            Statement::Expression(..) => return Vec::new(),
        };
        let Expression::BinaryCommand(BinaryCommand::Add, lhs, rhs, _) = expression else {
            return Vec::new();
        };
        // the variable being assigned has to be the one being added to
        let Expression::Variable(name, _) = &**lhs else {
            return Vec::new();
        };
        if name != assigned {
            return Vec::new();
        }
        // an array literal is the only right hand side that is certainly an array,
        // a variable could just as easily be a number or a string
        let (Expression::Array(elements, _) | Expression::ConsumeableArray(elements, _)) = &**rhs
        else {
            return Vec::new();
        };

        // a single element does not need a temporary array built for it
        let (command, value) = if let [element] = &elements[..] {
            ("pushBack", element.source(true))
        } else {
            ("append", rhs.source(true))
        };

        vec![Arc::new(CodeS45ArrayAppend::new(
            range.clone(),
            name.clone(),
            command,
            value,
            processed,
            config.severity(),
        ))]
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeS45ArrayAppend {
    span: Range<usize>,
    array: String,
    command: &'static str,
    value: String,
    severity: Severity,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeS45ArrayAppend {
    fn ident(&self) -> &'static str {
        "L-S45"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/sqf.html#array_append")
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> String {
        format!("Use `{}` instead of `array = array + [...]`", self.command)
    }

    fn label_message(&self) -> String {
        "array is copied to extend it".to_string()
    }

    fn note(&self) -> Option<String> {
        Some(format!(
            "`{}` extends the array in place, so anything else holding a reference to it will see the new elements",
            self.command
        ))
    }

    fn suggestion(&self) -> Option<String> {
        Some(format!("{} {} {}", self.array, self.command, self.value))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeS45ArrayAppend {
    #[must_use]
    pub fn new(
        span: Range<usize>,
        array: String,
        command: &'static str,
        value: String,
        processed: &Processed,
        severity: Severity,
    ) -> Self {
        Self {
            span,
            array,
            command,
            value,
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
