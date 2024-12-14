use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity},
};

use crate::{analyze::SqfLintData, Class, Config, Ident, Item, Property, Str, Value};

crate::analyze::lint!(LintC09MagwellMissingMagazine);

impl Lint<SqfLintData> for LintC09MagwellMissingMagazine {
    fn ident(&self) -> &'static str {
        "magwell_missing_magazine"
    }

    fn sort(&self) -> u32 {
        90
    }

    fn description(&self) -> &'static str {
        "Reports on magazines that are defined in CfgMagazineWells but not in CfgMagazines"
    }

    fn documentation(&self) -> &'static str {
r#"### Example

**Incorrect**
```hpp
class CfgMagazineWells {
    class abe_banana_shooter {
        abe_main[] = {
            "abe_cavendish",
            "abe_plantain",
            "external_banana"
        };
    };
};
class CfgMagazines {
    class abe_cavendish {};
};
```

**Correct**
```hpp
class CfgMagazineWells {
    class abe_banana_shooter {
        abe_main[] = {
            "abe_cavendish",
            "abe_plantain",
            "external_banana"
        };
    };
};
class CfgMagazines {
    class abe_cavendish {};
    class abe_plantain {};
};
```

### Explanation

Magazines defined in `CfgMagazineWells` that are using the project's prefix (abe in this case) must be defined in `CfgMagazines` as well. This is to prevent accidental typos or forgotten magazines.
"#

    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn minimum_severity(&self) -> Severity {
        Severity::Warning
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<SqfLintData>>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner<SqfLintData> for Runner {
    type Target = Config;
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Config,
        _data: &SqfLintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return vec![];
        };
        let mut codes: Codes = Vec::new();
        let mut classes = Vec::new();
        let Some(Property::Class(Class::Local {
            properties: magwells,
            ..
        })) = target
            .0
            .iter()
            .find(|p| p.name().value.to_lowercase() == "cfgmagazinewells")
        else {
            return codes;
        };
        let Some(Property::Class(Class::Local {
            properties: magazines,
            ..
        })) = target
            .0
            .iter()
            .find(|p| p.name().value.to_lowercase() == "cfgmagazines")
        else {
            return codes;
        };
        for property in magazines {
            if let Property::Class(Class::Local { name, .. }) = property {
                classes.push(name);
            }
        }
        for magwell in magwells {
            let Property::Class(Class::Local {
                properties: addons, ..
            }) = magwell
            else {
                continue;
            };
            for addon in addons {
                let Property::Entry {
                    name,
                    value: Value::Array(magazines),
                    ..
                } = addon
                else {
                    continue;
                };
                for mag in &magazines.items {
                    let Item::Str(Str { value, span }) = mag else {
                        continue;
                    };
                    if let Some(project) = project {
                        if !value
                            .to_lowercase()
                            .starts_with(&project.prefix().to_lowercase())
                        {
                            continue;
                        }
                    }
                    if !classes.iter().any(|c| c.value == *value) {
                        codes.push(Arc::new(Code09MagwellMissingMagazine::new(
                            name.clone(),
                            span.clone(),
                            processed,
                        )));
                    }
                }
            }
        }
        codes
    }
}

pub struct Code09MagwellMissingMagazine {
    array: Ident,
    span: Range<usize>,

    diagnostic: Option<Diagnostic>,
}

// TODO: maybe we could have a `did you mean` here without too much trouble?

impl Code for Code09MagwellMissingMagazine {
    fn ident(&self) -> &'static str {
        "L-C09"
    }

    fn link(&self) -> Option<&str> {
        Some("/analysis/config.html#magwell_missing_magazine")
    }

    fn message(&self) -> String {
        "magazine defined in CfgMagazineWells was not found in CfgMagazines".to_string()
    }

    fn label_message(&self) -> String {
        "no matching magazine was found".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code09MagwellMissingMagazine {
    #[must_use]
    pub fn new(array: Ident, span: Range<usize>, processed: &Processed) -> Self {
        Self {
            array,
            span,

            diagnostic: None,
        }
        .diagnostic_generate_processed(processed)
    }

    fn diagnostic_generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::from_code_processed(&self, self.span.clone(), processed);
        if let Some(diag) = &mut self.diagnostic {
            diag.labels.push({
                let Some(map) = processed.mapping(self.array.span.start) else {
                    return self;
                };
                Label::secondary(map.original().path().clone(), map.original().span())
                    .with_message("magazine well defined here")
            });
        }
        self
    }
}
