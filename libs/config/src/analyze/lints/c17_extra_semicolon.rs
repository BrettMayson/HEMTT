use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed},
};

use crate::{analyze::LintData, Property};

crate::analyze::lint!(LintC17ExtraSemicolon);

impl Lint<LintData> for LintC17ExtraSemicolon {
    fn ident(&self) -> &'static str {
        "extra_semicolon"
    }

    fn sort(&self) -> u32 {
        170
    }

    fn description(&self) -> &'static str {
        "Reports on extra semicolons after properties."
    }

    fn documentation(&self) -> &'static str {
"### Example

**Incorrect**
```hpp
class MyClass {
    a = 1;;
    b = 2;
};
```

**Correct**
```hpp
class MyClass {
    a = 1;
    b = 2;
};
```

**Incorrect**
```hpp
class MyClass {
    data = 1;
};;
```

**Correct**
```hpp
class MyClass {
    data = 1;
};
```

### Explanation

Extra semicolons after properties are not allowed in config files. This lint identifies and reports any instances where an extra semicolon is found after a property definition.
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
        if let Property::ExtraSemicolon(_, span) = target {
            vec![Arc::new(Code17ExtraSemicolon::new(
                span.clone(),
                processed,
            ))]
        } else {
            vec![]
        }
    }
}

pub struct Code17ExtraSemicolon {
    span: Range<usize>,
    diagnostic: Option<Diagnostic>,
    multiple: bool,
}

impl Code for Code17ExtraSemicolon {
    fn ident(&self) -> &'static str {
        "L-C17"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#extra_semicolon")
    }

    fn message(&self) -> String {
        if self.multiple {
            "property has multiple extra semicolons".to_string()
        } else {
            "property has an extra semicolon".to_string()
        }
    }

    fn label_message(&self) -> String {
        if self.multiple {
            "extra semicolons".to_string()
        } else {
            "extra semicolon".to_string()
        }
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "remove the extra semicolon{} from the property",
            if self.multiple { "s" } else { "" },
        ))
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code17ExtraSemicolon {
    #[must_use]
    pub fn new(span: Range<usize>, processed: &Processed) -> Self {
        let multiple = (span.end - span.start) > 1;
        Self {
            span,
            diagnostic: None,
            multiple,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        self
    }
}
