use std::sync::Arc;

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Processed},
};

use crate::{analyze::LintData, Class, Config, Property};

crate::analyze::lint!(LintC19StandardCase);

impl Lint<LintData> for LintC19StandardCase {
    fn ident(&self) -> &'static str {
        "standard_case"
    }

    fn sort(&self) -> u32 {
        190
    }

    fn description(&self) -> &'static str {
        "Reports on standard Arma config classes that do not use the correct casing"
    }

    fn documentation(&self) -> &'static str {
        r"### Example

**Incorrect**
```hpp
class cfgvehicles {
    class MyVehicle {};
};
```

**Correct**
```hpp
class CfgVehicles {
    class MyVehicle {};
};
```

### Explanation

Arma has several standard configuration classes that should follow a specific naming convention. While Arma is case-insensitive for class names, HEMTT enforces consistent casing for known standard classes to maintain code quality and readability.
"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(Runner)]
    }
}

const STANDARD_CLASSES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/c19_classes.rs"));

struct Runner;
impl LintRunner<LintData> for Runner {
    type Target = Config;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        _runtime: &hemtt_common::config::RuntimeArguments,
        target: &Config,
        _data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return vec![];
        };
        check(&target.0, processed)
    }
}

fn check(properties: &[Property], processed: &Processed) -> Codes {
    let mut codes = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    codes.extend(check(properties, processed));
                }
                Class::External { name } => {
                    if let Some(correct_case) =
                        STANDARD_CLASSES.iter().find(|&&class| {
                            class.eq_ignore_ascii_case(&name.value)
                        })
                        && &name.value != correct_case {
                            codes.push(Arc::new(CodeC19StandardCase::new(
                                c.clone(),
                                correct_case.to_string(),
                                processed,
                            )));
                        }
                }
                Class::Local {
                    name,
                    parent: _,
                    properties,
                    err_missing_braces: _,
                } => {
                    if let Some(correct_case) =
                        STANDARD_CLASSES.iter().find(|&&class| {
                            class.eq_ignore_ascii_case(&name.value)
                        })
                        && &name.value != correct_case {
                            codes.push(Arc::new(CodeC19StandardCase::new(
                                c.clone(),
                                correct_case.to_string(),
                                processed,
                            )));
                        }
                    codes.extend(check(properties, processed));
                }
            }
        }
    }
    codes
}

pub struct CodeC19StandardCase {
    class: Class,
    correct_case: String,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC19StandardCase {
    fn ident(&self) -> &'static str {
        "L-C19"
    }

    fn link(&self) -> Option<&str> {
        Some("/lints/config.html#standard_case")
    }

    fn message(&self) -> String {
        format!(
            "class `{}` should use standard casing: `{}`",
            self.class.name().map_or("unknown", crate::Ident::as_str),
            self.correct_case
        )
    }

    fn label_message(&self) -> String {
        format!("should be `{}`", self.correct_case)
    }

    fn help(&self) -> Option<String> {
        Some(format!(
            "rename this class to `{}`",
            self.correct_case
        ))
    }

    fn suggestion(&self) -> Option<String> {
        Some(self.correct_case.clone())
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC19StandardCase {
    #[must_use]
    pub fn new(class: Class, correct_case: String, processed: &Processed) -> Self {
        Self {
            class,
            correct_case,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        if let Some(name) = self.class.name() {
            self.diagnostic = Diagnostic::from_code_processed(&self, name.span.clone(), processed);
        }
        self
    }
}
