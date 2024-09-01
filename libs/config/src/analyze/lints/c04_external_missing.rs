use std::{collections::HashSet, sync::Arc};

use hemtt_common::config::{LintConfig, ProjectConfig};
use hemtt_workspace::{
    lint::{AnyLintRunner, Lint, LintRunner},
    reporting::{Code, Diagnostic, Processed},
};

use crate::{Class, Config, Property};

#[allow(clippy::module_name_repetitions)]
pub struct LintC04ExternalMissing;

impl Lint for LintC04ExternalMissing {
    fn ident(&self) -> &str {
        "external_missing"
    }

    fn description(&self) -> &str {
        "External class is missing"
    }

    fn documentation(&self) -> &str {
        "The external class is missing"
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
    ) -> Vec<std::sync::Arc<dyn Code>> {
        let Some(processed) = processed else {
            return vec![];
        };
        check(&target.0, &mut HashSet::new(), processed)
    }
}

fn check(
    properties: &[Property],
    defined: &mut HashSet<String>,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut codes: Vec<Arc<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    codes.extend(check(properties, defined, processed));
                }
                Class::External { name } => {
                    let name = name.value.to_lowercase();
                    defined.insert(name);
                }
                Class::Local {
                    name,
                    parent,
                    properties,
                } => {
                    let name = name.value.to_lowercase();
                    if let Some(parent) = parent {
                        let parent = parent.value.to_lowercase();
                        if parent != name && !defined.contains(&parent) {
                            codes.push(Arc::new(CodeC04ExternalMissing::new(c.clone(), processed)));
                        }
                    }
                    defined.insert(name);
                    codes.extend(check(properties, defined, processed));
                }
            }
        }
    }
    codes
}

#[allow(clippy::module_name_repetitions)]
pub struct CodeC04ExternalMissing {
    class: Class,
    diagnostic: Option<Diagnostic>,
}

impl Code for CodeC04ExternalMissing {
    fn ident(&self) -> &'static str {
        "L-C04"
    }

    fn message(&self) -> String {
        "class's parent is not present".to_string()
    }

    fn label_message(&self) -> String {
        "not present in config".to_string()
    }

    fn help(&self) -> Option<String> {
        self.class.parent().map(|parent| {
            format!(
                "add `class {};` to the config to declare it as external",
                parent.as_str(),
            )
        })
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        self.diagnostic.clone()
    }
}

impl CodeC04ExternalMissing {
    pub fn new(class: Class, processed: &Processed) -> Self {
        Self {
            class,
            diagnostic: None,
        }
        .generate_processed(processed)
    }

    fn generate_processed(mut self, processed: &Processed) -> Self {
        let Some(parent) = self.class.parent() else {
            panic!("CodeC04ExternalMissing::generate_processed called on class without parent");
        };
        self.diagnostic = Diagnostic::new_for_processed(&self, parent.span.clone(), processed);
        self
    }
}
