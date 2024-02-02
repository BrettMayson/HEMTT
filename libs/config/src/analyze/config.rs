use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_common::reporting::{Code, Processed};

use crate::{Class, Config, Ident, Item, Property, Str, Value};

use super::{
    codes::{
        ce3_duplicate_property::DuplicateProperty, ce7_missing_parent::MissingParent,
        cw1_parent_case::ParentCase, cw2_magwell_missing_magazine::MagwellMissingMagazine,
    },
    Analyze,
};

impl Analyze for Config {
    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Arc<dyn Code>> {
        let mut warnings = self
            .0
            .iter()
            .flat_map(|p| p.warnings(project, processed))
            .collect::<Vec<_>>();
        let mut defined = HashMap::new();
        warnings.extend(external_parent_case_warn(&self.0, &mut defined, processed));
        if let Some(project) = project {
            warnings.extend(magwell_missing_magazine(project, self, processed));
        }
        warnings
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>> {
        let mut errors = self
            .0
            .iter()
            .flat_map(|p| p.errors(project, processed))
            .collect::<Vec<_>>();
        let mut defined = HashSet::new();
        errors.extend(external_missing_error(&self.0, &mut defined, processed));
        errors.extend(duplicate_properties(&self.0, processed));
        errors
    }
}

fn external_missing_error(
    properties: &[Property],
    defined: &mut HashSet<String>,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    errors.extend(external_missing_error(properties, defined, processed));
                }
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
                            errors.push(Arc::new(MissingParent::new(c.clone(), processed)));
                        }
                    }
                    defined.insert(name);
                    errors.extend(external_missing_error(properties, defined, processed));
                }
            }
        }
    }
    errors
}

fn external_parent_case_warn(
    properties: &[Property],
    defined: &mut HashMap<String, Class>,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut warnings: Vec<Arc<dyn Code>> = Vec::new();
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
                                    warnings.push(Arc::new(ParentCase::new(
                                        c.clone(),
                                        parent_class.clone(),
                                        processed,
                                    )));
                                }
                            }
                        } else if parent.value != name.value {
                            warnings.push(Arc::new(ParentCase::new(
                                c.clone(),
                                c.clone(),
                                processed,
                            )));
                        }
                    }
                    defined.insert(name_lower, c.clone());
                    warnings.extend(external_parent_case_warn(properties, defined, processed));
                }
            }
        }
    }
    warnings
}

fn duplicate_properties(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut seen: HashMap<String, Vec<(bool, Ident)>> = HashMap::new();
    duplicate_properties_inner("", properties, &mut seen);
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    for (_, idents) in seen {
        if idents.len() > 1 && !idents.iter().all(|(class, _)| *class) {
            errors.push(Arc::new(DuplicateProperty::new(
                idents.into_iter().map(|(_, i)| i).collect(),
                processed,
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

fn magwell_missing_magazine(
    project: &ProjectConfig,
    config: &Config,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut warnings: Vec<Arc<dyn Code>> = Vec::new();
    let mut classes = Vec::new();
    let Some(Property::Class(Class::Local {
        properties: magwells,
        ..
    })) = config
        .0
        .iter()
        .find(|p| p.name().value.to_lowercase() == "cfgmagazinewells")
    else {
        return warnings;
    };
    let Some(Property::Class(Class::Local {
        properties: magazines,
        ..
    })) = config
        .0
        .iter()
        .find(|p| p.name().value.to_lowercase() == "cfgmagazines")
    else {
        return warnings;
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
                    warnings.push(Arc::new(MagwellMissingMagazine::new(
                        name.clone(),
                        span.clone(),
                        processed,
                    )));
                }
            }
        }
    }
    warnings
}
