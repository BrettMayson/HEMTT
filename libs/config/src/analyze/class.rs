use std::collections::HashMap;

use hemtt_error::{processed::Processed, Code};

use crate::{Class, Ident};

use super::{codes::ce3_duplicate_property::DuplicateProperty, Analyze};

impl Analyze for Class {
    fn valid(&self) -> bool {
        match self {
            Self::External { .. } => true,
            Self::Local { properties, .. } => properties.iter().all(Analyze::valid),
        }
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => properties
                .iter()
                .flat_map(|p| p.warnings(processed))
                .collect::<Vec<_>>(),
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => {
                let mut errors = properties
                    .iter()
                    .flat_map(|p| p.errors(processed))
                    .collect::<Vec<_>>();
                errors.extend(self.duplicate_properties());
                errors
            }
        }
    }
}

impl Class {
    fn duplicate_properties(&self) -> Vec<Box<dyn Code>> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } => {
                let mut errors: Vec<Box<dyn Code>> = Vec::new();
                let mut seen = Vec::new();
                let mut conflicts = HashMap::new();
                for property in properties {
                    if let Some(b) = seen
                        .iter()
                        .find(|b: &&Ident| b.value == property.name().value)
                    {
                        conflicts
                            .entry(b.as_str().to_string())
                            .or_insert_with(|| vec![b.clone()])
                            .push(property.name().clone());
                        continue;
                    }
                    seen.push(property.name().clone());
                }
                for (_, conflict) in conflicts {
                    errors.push(Box::new(DuplicateProperty::new(conflict)));
                }
                errors
            }
        }
    }
}
