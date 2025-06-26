use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{diagnostic::Yellow, Code, Diagnostic, Processed},
};

use crate::{analyze::LintData, Property};

crate::analyze::lint!(LintC08MissingSemicolon);

impl Lint<LintData> for LintC08MissingSemicolon {
    fn ident(&self) -> &'static str {
        "missing_semicolon"
    }

    fn sort(&self) -> u32 {
        80
    }

    fn description(&self) -> &'static str {
        "Reports on properties that are missing a semicolon"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```hpp
class MyClass {
    data = 1
};
```

**Correct**
```hpp
class MyClass {
    data = 1;
};
```

**Incorrect**
```hpp
class MyClass {
    data = 1;
}
```

**Correct**
```hpp
class MyClass {
    data = 1;
};
```

### Explanation

All properties must end with a semicolon, including classes.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::fatal()
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
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &crate::Property,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        if let Property::MissingSemicolon(_, span) = target {
            vec![Arc::new(Code08MissingSemicolon::new(
                span.clone(),
                processed,
            ))]
        } else {
            vec![]
        }
    }
}

pub struct Code08MissingSemicolon {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code08MissingSemicolon {
    fn ident(&self) -> &'static str {
        "L-C08"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#missing_semicolon")
    }

    fn message(&self) -> String {
        "property is missing a semicolon".to_string()
    }

    fn label_message(&self) -> String {
        "missing semicolon".to_string()
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "add a semicolon {} to the end of the property",
            Yellow.paint(";")
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code08MissingSemicolon {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        Self {
            span,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let haystack = &processed.extract(self.span.clone());
        let possible_end = self.span.start
            + haystack
                .find('\n')
                .unwrap_or_else(|| haystack.rfind(|c: char| c != ' ' && c != '}').unwrap_or(0) + 1);
        self.diagnostic =
            Diagnostic::from_code_processed(&self, possible_end..possible_end, processed);
        self
    }
}
