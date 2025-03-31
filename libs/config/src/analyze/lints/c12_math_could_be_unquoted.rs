use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed, Severity},
};

use crate::{analyze::LintData, Item, Number, Property, Value};

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
        r#"### Configuration

- **ignore**: Specifies a list of properties to ignore, typically because they may contain false positives.
- **forced**: Specifies a boolean to check all properites for numbers, or list of properties that should be checked to be numbers.

**default values shown below**

```toml
[lints.config.math_could_be_unquoted]
options.ignore = ["text", "name", "displayname", "icontext"]
options.forced = ["initspeed", "ambient", "diffuse", "forceddiffuse", "emmisive", "specular", "specularpower"]
```

### Example

**Incorrect**
```hpp
x = '1+1';
```

**Correct**
```hpp
x = 1+1; // HEMTT will evaluate at build-time to 2
```

### Explanation
Quoted math statements will have to be evaulated on each use in-game, by allowing HEMTT to evaluate the math at build-time you can save some performance."#
    }

    fn default_config(&self) -> LintConfig {
        // false-positives are possible - pabst
        // I think this will be rare enough that people can ignore the help,
        // imo the value in this being on by default is worth the false positives - brett
        LintConfig::help()
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
        config: &LintConfig,
        processed: Option<&Processed>,
        target: &crate::Property,
        _data: &LintData,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let mut codes = Vec::new();
        let Some(processed) = processed else {
            return vec![];
        };
        let Property::Entry { name, value, .. } = target else {
            return vec![];
        };
        let name = name.as_str().to_lowercase();
        let ignore = if let Some(toml::Value::Array(ignore)) = config.option("ignore") {
            ignore.iter().map(|v| v.as_str().expect("ignore items must be strings")).collect::<Vec<&str>>()
        } else {
            vec!["text", "name", "displayname", "icontext"]
        };
        if ignore.iter().any(|s| s.to_lowercase() == name) {
            return vec![];
        }
        let check_if_equation = !match config.option("forced") {
            Some(toml::Value::Boolean(forced)) => *forced,
            Some(toml::Value::Array(forced)) => forced.iter().map(|v| v.as_str().expect("forced items must be strings").to_lowercase()).any(|x| x == name.as_str()),
            None => ["initspeed", "ambient", "diffuse", "forceddiffuse", "emmisive", "specular", "specularpower"].contains(&name.as_str()),
            _ => {
                println!("Invalid forced value on math_could_be_unquoted, expected boolean or array of strings");
                false
            }
        };
        match value {
            Value::Array(arr) => {
                for item in &arr.items {
                    check_item(item, processed, config, check_if_equation, &mut codes);
                }
            }
            Value::Str(str) => check_str(str, processed, config, check_if_equation, &mut codes),
            _ => {}
        }

        codes
    }
}

fn check_item(
    target: &crate::Item,
    processed: &Processed,
    config: &LintConfig,
    check_if_equation: bool,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    match target {
        Item::Array(items) => {
            for element in items {
                check_item(element, processed, config, check_if_equation, codes);
            }
        }
        Item::Str(taget_str) => {
            check_str(taget_str, processed, config, check_if_equation, codes);
        }
        _ => {}
    }
}

fn check_str(
    target_str: &crate::Str,
    processed: &Processed,
    config: &LintConfig,
    check_if_equation: bool,
    codes: &mut Vec<Arc<dyn Code>>,
) {
    let raw_string = target_str.value();
    // check if it contains some kind of math ops (avoid false positives from `displayName = "556";`)
    if check_if_equation && !(raw_string.contains('+')
        || raw_string.contains('-')
        || raw_string.contains('*')
        || raw_string.contains('/'))
    {
        return;
    }
    // attempt to parse it as a number
    let Some(num) = Number::try_evaulation(raw_string, target_str.span()) else {
        return;
    };
    let span = target_str.span().start + 1..target_str.span().end - 1;
    codes.push(Arc::new(Code12MathCouldBeUnquoted::new(
        span,
        processed,
        format!("reducible to: {num}"),
        config.severity(),
    )));
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
