use std::collections::{HashMap, HashSet};

use hemtt_common::reporting::{Code, Processed};

use crate::{Class, Config, Property};

use super::{
    codes::{ce7_missing_parent::MissingParent, cw1_parent_case::ParentCase},
    Analyze,
};

impl Analyze for Config {
    fn valid(&self) -> bool {
        self.0.iter().all(Analyze::valid)
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        let mut warnings = self
            .0
            .iter()
            .flat_map(|p| p.warnings(processed))
            .collect::<Vec<_>>();
        let mut defined = HashMap::new();
        warnings.extend(external_missing_warn(&self.0, &mut defined));
        warnings
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        let mut errors = self
            .0
            .iter()
            .flat_map(|p| p.errors(processed))
            .collect::<Vec<_>>();
        let mut defined = HashSet::new();
        errors.extend(external_missing_error(&self.0, &mut defined));
        errors
    }
}

fn external_missing_error(
    properties: &[Property],
    defined: &mut HashSet<String>,
) -> Vec<Box<dyn Code>> {
    let mut errors: Vec<Box<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::External { name } => {
                    let name = name.value.to_lowercase();
                    if !defined.contains(&name) {
                        defined.insert(name);
                    }
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
                            errors.push(Box::new(MissingParent::new(c.clone())));
                        }
                    }
                    defined.insert(name);
                    errors.extend(external_missing_error(properties, defined));
                }
            }
        }
    }
    errors
}

fn external_missing_warn(
    properties: &[Property],
    defined: &mut HashMap<String, Class>,
) -> Vec<Box<dyn Code>> {
    let mut warnings: Vec<Box<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::External { name } => {
                    let name = name.value.to_lowercase();
                    defined.entry(name).or_insert_with(|| c.clone());
                }
                Class::Local {
                    name,
                    parent,
                    properties,
                } => {
                    let name = name.value.to_lowercase();
                    if let Some(parent) = parent {
                        let parent_lower = parent.value.to_lowercase();
                        if parent_lower != name {
                            if let Some(parent_class) = defined.get(&parent_lower) {
                                if parent_class.name().value != parent.value {
                                    warnings.push(Box::new(ParentCase::new(
                                        c.clone(),
                                        parent_class.clone(),
                                    )));
                                }
                            }
                        }
                    }
                    defined.insert(name, c.clone());
                    warnings.extend(external_missing_warn(properties, defined));
                }
            }
        }
    }
    warnings
}
