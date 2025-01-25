use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed},
};

use crate::{analyze::LintData, Class, Property};

crate::analyze::lint!(LintC10ClassMissingBraces);

impl Lint<LintData> for LintC10ClassMissingBraces {
    fn ident(&self) -> &'static str {
        "class_missing_braces"
    }

    fn sort(&self) -> u32 {
        100
    }

    fn description(&self) -> &'static str {
        "Reports on classes that use inheritance without braces"
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```hpp
class External;
class AlsoExternal: External;
class MyClass: AlsoExternal {
    data = 1;
};
```

**Correct**
```hpp
class External;
class AlsoExternal: External {};
class MyClass: AlsoExternal {
    data = 1;
};
```

### Explanation

All classes using inheritance with a parent class must use braces `{}`, even if the class has no properties.
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
        if let Property::Class(Class::Local { err_missing_braces, parent, .. }) = target {
            if *err_missing_braces {
                return vec![Arc::new(Code10ClassMissingBraces::new(
                    parent.clone().expect("parent must be present for err_missing_braces").span,
                    processed,
                ))];
            }
        }
        vec![]
    }
}

pub struct Code10ClassMissingBraces {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code10ClassMissingBraces {
    fn ident(&self) -> &'static str {
        "L-C10"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#class_missing_braces")
    }

    fn message(&self) -> String {
        "classes must use braces when inheriting".to_string()
    }

    fn label_message(&self) -> String {
        "missing braces".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(" {};".to_string())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code10ClassMissingBraces {
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
