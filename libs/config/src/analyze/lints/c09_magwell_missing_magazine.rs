use std::{ops::Range, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed},
};

use crate::{Class, Config, Ident, Item, Property, Str, Value};

#[allow(clippy::module_name_repetitions)]
pub struct LintC09MagwellMissingMagazine;

impl Lint for LintC09MagwellMissingMagazine {
    fn ident(&self) -> &str {
        "magwell_missing_magazine"
    }

    fn description(&self) -> &str {
        "Magwell missing magazine"
    }

    fn documentation(&self) -> &str {
        "The magwell is missing a magazine"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::error()
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner for Runner {
    type Target = Config;
    fn run_processed(
        &self,
        project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: &Processed,
        target: &Config,
    ) -> Vec<Arc<dyn Code>> {
        let Some(project) = project else {
            return vec![];
        };
        let mut codes: Vec<Arc<dyn Code>> = Vec::new();
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
                    if !value
                        .to_lowercase()
                        .starts_with(&project.prefix().to_lowercase())
                    {
                        continue;
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
        "CW2"
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
    pub fn new(array: Ident, span: Range<usize>, processed: &Processed) -> Self {
        Self {
            array,
            span,

            diagnostic: None,
        }
        .diagnostic_generate_processed(processed)
    }

    fn diagnostic_generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(&self, self.span.clone(), processed);
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
