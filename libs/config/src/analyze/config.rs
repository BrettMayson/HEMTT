use std::collections::{HashMap, HashSet};

use hemtt_common::reporting::{Code, Processed};

use crate::{Class, Config, Ident, Property};

use super::{
    codes::{
        ce3_duplicate_property::DuplicateProperty, ce7_missing_parent::MissingParent,
        cw1_parent_case::ParentCase,
    },
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
        warnings.extend(external_parent_case_warn(&self.0, &mut defined));
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
        errors.extend(duplicate_properties(&self.0));
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

fn external_parent_case_warn(
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
                    let name_lower = name.value.to_lowercase();
                    if let Some(parent) = parent {
                        let parent_lower = parent.value.to_lowercase();
                        if parent_lower != name_lower {
                            if let Some(parent_class) = defined.get(&parent_lower) {
                                if parent_class.name().value != parent.value {
                                    warnings.push(Box::new(ParentCase::new(
                                        c.clone(),
                                        parent_class.clone(),
                                    )));
                                }
                            }
                        } else if parent.value != name.value {
                            warnings.push(Box::new(ParentCase::new(c.clone(), c.clone())));
                        }
                    }
                    defined.insert(name_lower, c.clone());
                    warnings.extend(external_parent_case_warn(properties, defined));
                }
            }
        }
    }
    warnings
}

fn duplicate_properties(properties: &[Property]) -> Vec<Box<dyn Code>> {
    let mut seen: HashMap<String, Vec<(bool, Ident)>> = HashMap::new();
    duplicate_properties_inner("", properties, &mut seen);
    let mut errors: Vec<Box<dyn Code>> = Vec::new();
    for (_, idents) in seen {
        if idents.len() > 1 && !idents.iter().all(|(class, _)| *class) {
            errors.push(Box::new(DuplicateProperty::new(
                idents.into_iter().map(|(_, i)| i).collect(),
            )));
        }
    }
    errors
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
