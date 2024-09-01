use std::{collections::HashMap, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Label, Processed, Severity},
};

use crate::{Class, Property};

#[allow(clippy::module_name_repetitions)]
pub struct LintC05ExternalParentCase;

impl Lint for LintC05ExternalParentCase {
    fn ident(&self) -> &str {
        "external_parent_case"
    }

    fn description(&self) -> &str {
        "External parent case"
    }

    fn documentation(&self) -> &str {
        "The external parent case is incorrect"
    }

    fn default_config(&self) -> LintConfig {
        LintConfig::warning()
    }

    fn minimum_severity(&self) -> Severity {
        Severity::Note
    }

    fn runners(&self) -> Vec<Box<dyn AnyLintRunner>> {
        vec![Box::new(Runner)]
    }
}

struct Runner;
impl LintRunner for Runner {
    type Target = Class;
    fn run(
        &self,
        _project: Option<&ProjectConfig>,
        _config: &LintConfig,
        processed: Option<&Processed>,
        target: &Class,
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        check(target.properties(), &mut HashMap::new(), processed)
    }
}

fn check(
    properties: &[Property],
    defined: &mut HashMap<String, Class>,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut codes: Vec<Arc<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { .. } => {
                    panic!("Root class should not be in the config");
                }
                Class::External { name } => {
                    let name = name.value.to_lowercase();
                    defined.entry(name).or_insert_with(|| c.clone());
                }
                Class::Local {
                    name,
                    parent,
                    properties,
                } => {
                    let name_lower = name.value.to_lowercase();
                    if let Some(parent) = parent {
                        let parent_lower = parent.value.to_lowercase();
                        if parent_lower != name_lower {
                            if let Some(parent_class) = defined.get(&parent_lower) {
                                if parent_class.name().map(|p| &p.value) != Some(&parent.value) {
                                    codes.push(Arc::new(Code05ExternalParentCase::new(
                                        c.clone(),
                                        parent_class.clone(),
                                        processed,
                                    )));
                                }
                            }
                        } else if parent.value != name.value {
                            codes.push(Arc::new(Code05ExternalParentCase::new(
                                c.clone(),
                                c.clone(),
                                processed,
                            )));
                        }
                    }
                    defined.insert(name_lower, c.clone());
                    codes.extend(check(properties, defined, processed));
                }
            }
        }
    }
    codes
}

pub struct Code05ExternalParentCase {
    class: Class,
    parent: Class,
    diagnostic: Option<Diagnostic>,
}

impl Code for Code05ExternalParentCase {
    fn ident(&self) -> &'static str {
        "L-C05"
    }

    fn message(&self) -> String {
        "parent case does not match parent definition".to_string()
    }

    fn label_message(&self) -> String {
        "parent does not match definition case".to_string()
    }

    fn help(&self) -> Option<String> {
        Some("change the parent case to match the parent definition".to_string())
    }

    fn suggestion(&self) -> Option<String> {
        Some(
            self.parent
                .name()
                .expect("parent existed to create error")
                .as_str()
                .to_string(),
        )
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl Code05ExternalParentCase {
    pub fn new(class: Class, parent: Class, processed: &Processed) -> Self {
        Self {
            class,
            parent,

            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        self.diagnostic = Diagnostic::new_for_processed(
            &self,
            self.class
                .parent()
                .expect("parent existed to create error")
                .span
                .clone(),
            processed,
        );
        if let Some(diag) = &mut self.diagnostic {
            let Some(parent) = self.class.parent() else {
                panic!(
                    "Code05ExternalParentCase::generate_processed called on class without parent"
                );
            };
            let map = processed
                .mapping(
                    self.parent
                        .name()
                        .expect("parent existed to create error")
                        .span
                        .start,
                )
                .expect("mapping should exist");
            let file = processed.source(map.source()).expect("source should exist");
            diag.labels.push(
                Label::secondary(
                    file.0.clone(),
                    map.original_start()..map.original_start() + parent.span.len(),
                )
                .with_message("parent definition here"),
            );
        }
        self
    }
}
