use std::{collections::HashMap, sync::Arc};

use hemtt_workspace::reporting::{Code, Processed};

use crate::{analyze::codes::ce3_duplicate_property::DuplicateProperty, Class, Ident, Property};

pub fn duplicate_properties(properties: &[Property], processed: &Processed) -> Vec<Arc<dyn Code>> {
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
