use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    addons::Addon,
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Codes, Diagnostic, Label, Processed, Severity},
};

use crate::{analyze::LintData, Class, Config, Ident, Item, Property, Str, Value};

crate::analyze::lint!(LintC09MagwellMissingMagazine);

impl Lint<LintData> for LintC09MagwellMissingMagazine {
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
        LintConfig::fatal()
    }

    fn minimum_severity(&self) -> Severity {
        Severity::Warning
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner<LintData>>> {
        vec![Box::new(RunnerScan), Box::new(RunnerFinal)]
    }
}

struct RunnerScan;
impl LintRunner<LintData> for RunnerScan {
    type Target = Config;
    fn run(
        &self,
        project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Config,
        data: &LintData,
    ) -> Codes {
        let Some(processed) = processed else {
            return vec![];
        };
        let mut codes = Vec::new();
        let mut classes = Vec::new();

        if let Some(Property::Class(Class::Local {
            properties: magazines,
            ..
        })) = target
            .0
            .iter()
            .find(|p| p.name().value.to_lowercase() == "cfgmagazines")
        {
            for property in magazines {
                if let Property::Class(Class::Local { name, .. }) = property {
                    classes.push(name);
                }
            }
        };

        if let Some(Property::Class(Class::Local {
            properties: magwells,
            ..
        })) = target
            .0
            .iter()
            .find(|p| p.name().value.to_lowercase() == "cfgmagazinewells")
        {
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
                            let code: Arc<dyn Code> = Arc::new(Code09MagwellMissingMagazine::new(
                                name.clone(),
                                span.clone(),
                                processed,
                            ));
                            codes.push((value.clone(), code));
                        }
                    }
                }
            }
        };
        {
        let mut magazine_well_error_info = data.magazine_well_info.lock().expect("mutex safety");
        magazine_well_error_info
            .0
            .extend(classes.iter().map(|i| i.as_str().to_string()));
        magazine_well_error_info.1.extend(codes);
        }
        vec![]
    }
}

/// Runner for finale during `pre_build2`
struct RunnerFinal;
impl LintRunner<LintData> for RunnerFinal {
    type Target = Vec<Addon>;

    fn run(
        &self,
        _project: Option<&hemtt_common::config::ProjectConfig>,
        _config: &LintConfig,
        _processed: Option<&hemtt_workspace::reporting::Processed>,
        target: &Self::Target,
        _data: &LintData,
    ) -> Codes {
        let mut all_magazines = Vec::new();
        let mut all_codes = Vec::new();

        for addon in target {
            let (mags, magwell_codes) = addon
                .build_data()
                .magazine_well_info()
                .lock()
                .expect("not poision")
                .clone();
            all_magazines.extend(mags);
            all_codes.extend(magwell_codes);
        }
        all_codes
            .iter()
            .filter(|(missing_mag, _)| !all_magazines.iter().any(|c| c == missing_mag))
            .map(|(_, code)| code.clone())
            .collect()
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
