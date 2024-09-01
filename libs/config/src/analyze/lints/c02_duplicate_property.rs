use std::{collections::HashMap, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed},
};

use crate::{Class, Config, Ident, Property};

#[allow(clippy::module_name_repetitions)]
pub struct LintC02DuplicateProperty;

impl Lint for LintC02DuplicateProperty {
    fn ident(&self) -> &str {
        "duplicate_property"
    }

    fn description(&self) -> &str {
        "Duplicate property"
    }

    fn documentation(&self) -> &str {
        "The property is duplicated"
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
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Config,
    ) -> Vec<Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        let mut seen: HashMap<String, Vec<(bool, Ident)>> = HashMap::new();
        duplicate_properties_inner("", &target.0, &mut seen);
        let mut codes: Vec<Arc<dyn Code>> = Vec::new();
        for (_, idents) in seen {
            if idents.len() > 1 && !idents.iter().all(|(class, _)| *class) {
                codes.push(Arc::new(CodeC02DuplicateProperty::new(
                    idents.iter().map(|(_, i)| i.clone()).collect(),
                    processed,
                )));
            }
        }
        codes
    }
}

fn duplicate_properties_inner(
    scope: &str,
    properties: &[Property],
    seen: &mut HashMap<String, Vec<(bool, Ident)>>,
) {
    for property in properties {
        match property {
            Property::Class(Class::Local {
                name, properties, ..
            }) => {
                duplicate_properties_inner(
                    &format!("{}.{}", scope, name.value.to_lowercase()),
                    properties,
                    seen,
                );
                let entry = seen
                    .entry(format!("{}.{}", scope, name.value.to_lowercase()))
                    .or_default();
                entry.push((true, name.clone()));
            }
            Property::Entry { name, .. } | Property::MissingSemicolon(name, _) => {
                let entry = seen
                    .entry(format!("{}.{}", scope, name.value.to_lowercase()))
                    .or_default();
                entry.push((false, name.clone()));
            }
            _ => (),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC02DuplicateProperty {
    conflicts: Vec<Ident>,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC02DuplicateProperty {
    fn ident(&self) -> &'static str {
        "L-C02"
    }

    fn message(&self) -> String {
        "property was defined more than once".to_string()
    }

    fn label_message(&self) -> String {
        "duplicate property".to_string()
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC02DuplicateProperty {
    pub fn new(conflicts: Vec<Ident>, processed: &Processed) -> Self {
        Self {
            conflicts,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            self.conflicts
                .last()
                .expect("conflicts should have at least one element if it was created with new")
                .span
                .clone(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            for conflict in self.conflicts.iter().rev().skip(1) {
                let map = processed
                    .mapping(conflict.span.start)
                    .expect("mapping should exist");
                let file = processed.source(map.source()).expect("source should exist");
                diag.labels.push(
                    Label::secondary(
                        file.0.clone(),
                        map.original_start()..map.original_start() + conflict.span.len(),
                    )
                    .with_message("also defined here"),
                );
            }
        }
        self
    }
}
