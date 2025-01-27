use std::sync::Arc;

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed},
};

use crate::{analyze::LintData, Property, Value};

crate::analyze::lint!(LintC06UnexpectedArray);

impl Lint<LintData> for LintC06UnexpectedArray {
    fn ident(&self) -> &'static str {
        "unexpected_array"
    }

    fn sort(&self) -> u32 {
        60
    }

    fn description(&self) -> &'static str {
        "Reports on properties that are not expected to be arrays, but are defined as arrays"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```hpp
class MyClass {
    data = {1, 2, 3};
};
```

**Correct**
```hpp
class MyClass {
    data[] = {1, 2, 3};
};
```

### Explanation

Arrays in Arma configs are denoted by `[]` after the property name.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = crate::Property;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &crate::Property,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        let Property::Entry {
            value: Value::UnexpectedArray(_),
            ..
        } = target
        else {
            return vec![];
        };
        vec![Arc::new(Code06UnexpectedArray::new(
            target.clone(),
            processed,
        ))]
    }
}

pub struct Code06UnexpectedArray {
    property: Property,
    diagnostic: Option<Diagnostic>,
    suggestion: Option<String>,
}

impl Code for Code06UnexpectedArray {
    fn ident(&self) -> &'static str {
        "L-C06"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#unexpected_array")
    }

    fn message(&self) -> String {
        "property was not expected to be an array".to_string()
    }

    fn label_message(&self) -> String {
        "expected [] here".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("add [] to the property".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        self.suggestion.clone()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code06UnexpectedArray {
    #[must_use]
    pub fn new(property: Property, processed: &Processed) -> Self {
        Self {
            property,
            diagnostic: None,
            suggestion: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Property::Entry {
            name,
            value: Value::UnexpectedArray(array),
            ..
        } = &self.property
        else {
            panic!("Code06UnexpectedArray::generate_processed called on non-Code06UnexpectedArray property");
        };
        let array_start = processed
            .mapping(array.span.start)
            .expect("mapping should exist");
        let array_file = processed
            .source(array_start.source())
            .expect("source should exist");
        let ident_start = processed
            .mapping(name.span.start)
            .expect("mapping should exist");
        let ident_end = processed
            .mapping(name.span.end)
            .expect("mapping should exist");
        self.suggestion = Some(format!("{}[]", name.value));
        self.diagnostic = Diagnostic::from_code_processed(
            &self,
            ident_start.original_start()..ident_end.original_start(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            diag.labels.push(
                Label::secondary(array_file.0.clone(), array.span.clone())
                    .with_message("unexpected array"),
            );
        }
        self
    }
}
