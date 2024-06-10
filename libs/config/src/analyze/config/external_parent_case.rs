use std::{collections::HashMap, sync::Arc};

use hemtt_workspace::reporting::{Code, Processed};

use crate::{analyze::codes::cw1_parent_case::ParentCase, Class, Property};

pub fn warn(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
    warn_inner(properties, &mut HashMap::new(), processed)
}

fn warn_inner(
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
                    warnings.extend(warn_inner(properties, defined, processed));
                }
            }
        }
    }
    warnings
}
