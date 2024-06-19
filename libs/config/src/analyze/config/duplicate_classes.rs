use std::{collections::HashMap, sync::Arc};

use hemtt_workspace::reporting::{Code, Processed};

use crate::{analyze::codes::ce8_duplicate_classes::DuplicateClasses, Class, Property};

pub fn error(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
    let mut defined: HashMap<String, Vec<Class>> = HashMap::new();
    let mut errors = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::Root { properties } => {
                    errors.extend(error(properties, processed));
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
                    errors.extend(error(properties, processed));
                    defined
                        .entry(name.value.to_lowercase())
                        .or_default()
                        .push(c.clone());
                }
            }
        }
    }
    errors.extend(defined.into_iter().filter_map(|(_, classes)| {
        if classes.len() > 1 {
            Some(Arc::new(DuplicateClasses::new(classes, processed)) as Arc<dyn Code>)
        } else {
            None
        }
    }));
    errors
}
