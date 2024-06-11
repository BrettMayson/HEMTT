use std::{collections::HashSet, sync::Arc};

use hemtt_workspace::reporting::{Code, Processed};

use crate::{analyze::codes::ce7_missing_parent::MissingParent, Class, Property};

pub fn error(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
    error_inner(properties, &mut HashSet::new(), processed)
}

fn error_inner(
    properties: &[Property],
    defined: &mut HashSet<String>,
    processed: &Processed,
) -> Vec<Arc<dyn Code>> {
    let mut errors: Vec<Arc<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    errors.extend(error_inner(properties, defined, processed));
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
                    errors.extend(error_inner(properties, defined, processed));
                }
            }
        }
    }
    errors
}
