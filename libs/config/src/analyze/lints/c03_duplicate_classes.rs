use std::{collections::HashMap, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed},
};

use crate::{Class, Config, Property};

#[allow(clippy::module_name_repetitions)]
pub struct LintC03DuplicateClasses;

impl Lint for LintC03DuplicateClasses {
    fn ident(&self) -> &str {
        "duplicate_classes"
    }

    fn description(&self) -> &str {
        "Duplicate classes"
    }

    fn documentation(&self) -> &str {
        "The class is duplicated"
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
        check(&target.0, processed)
    }
}

pub fn check(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut defined: HashMap<String, Vec<Class>> = HashMap::new();
    let mut codes = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    codes.extend(check(properties, processed));
                }
                Class::External { name } => {
                    defined
                        .entry(name.value.to_lowercase())
                        .or_default()
                        .push(c.clone());
                }
                Class::Local {
                    name,
                    parent: _,
                    properties,
                } => {
                    codes.extend(check(properties, processed));
                    defined
                        .entry(name.value.to_lowercase())
                        .or_default()
                        .push(c.clone());
                }
            }
        }
    }
    codes.extend(defined.into_iter().filter_map(|(_, classes)| {
        if classes.len() > 1 {
            Some(Arc::new(CodeC03DuplicateClasses::new(classes, processed)) as Arc<dyn Code>)
        } else {
            None
        }
    }));
    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC03DuplicateClasses {
    classes: Vec<Class>,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC03DuplicateClasses {
    fn ident(&self) -> &'static str {
        "L-C03"
    }

    fn message(&self) -> String {
        "class defined multiple times".to_string()
    }

    fn label_message(&self) -> String {
        "defined multiple times".to_string()
    }

    fn help(&self) -> Option<String> {
        self.classes
            .first()
            .expect("at least one class")
            .name()
            .map(|parent| {
                format!(
                    "remove all but the first definition of `class {};`",
                    parent.as_str(),
                )
            })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC03DuplicateClasses {
    pub fn new(classes: Vec<Class>, processed: &Processed) -> Self {
        Self {
            classes,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(name) = self.classes[0].name() else {
            panic!("CodeC03DuplicateClasses::generate_processed called on class without name");
        };
        self.diagnostic = Diagnostic::new_for_processed(&self, name.span.clone(), processed);
        if let Some(diag) = &mut self.diagnostic {
            for class in self.classes.iter().skip(1) {
                let map = processed
                    .mapping(class.name().expect("class should have name").span.start)
                    .expect("mapping should exist");
                let file = processed.source(map.source()).expect("source should exist");
                diag.labels.push(
                    Label::secondary(
                        file.0.clone(),
                        map.original_start()
                            ..map.original_start()
                                + class.name().expect("class should have name").span.len(),
                    )
                    .with_message("also defined here"),
                );
            }
        }
        self
    }
}
