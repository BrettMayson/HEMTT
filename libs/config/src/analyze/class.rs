use std::collections::HashMap;

use hemtt_common::error::{processed::Processed, Code};

use crate::{Class, Ident, Property};

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
                let mut seen: HashMap<Ident, &Property> = HashMap::new();
                let mut conflicts = HashMap::new();
                for property in properties {
                    if matches!(
                        property,
                        Property::Delete(_) | Property::Class(Self::External { .. })
                    ) {
                        continue;
                    }
                    if let Some(b) = seen.iter().find(|(b, _)| b.value == property.name().value) {
                        if let Property::Class(a) = property {
                            if let Property::Class(ib) = b.1 {
                                errors.extend(a.duplicate_inner(ib));
                                continue;
                            }
                        }
                        conflicts
                            .entry(b.0.as_str().to_string())
                            .or_insert_with(|| vec![b.0.clone()])
                            .push(property.name().clone());
                        continue;
                    }
                    seen.insert(property.name().clone(), property);
                }
                for (_, conflict) in conflicts {
                    errors.push(Box::new(DuplicateProperty::new(conflict)));
                }
                errors
            }
        }
    }

    fn duplicate_inner(&self, other: &Self) -> Vec<Box<dyn Code>> {
        let Self::Local { properties: a, .. } = self else {
            return vec![];
        };
        let Self::Local { properties: b, .. } = other else {
            return vec![];
        };

        let mut errors: Vec<Box<dyn Code>> = Vec::new();
        for a in a {
            if let Property::Class(a) = a {
                if let Some(Property::Class(b)) =
                    b.iter().find(|b| b.name().as_str() == a.name().as_str())
                {
                    errors.extend(a.duplicate_inner(b));
                    continue;
                }
            }
            if let Some(b) = b.iter().find(|b| b.name().as_str() == a.name().as_str()) {
                errors.push(Box::new(DuplicateProperty::new(vec![
                    b.name().clone(),
                    a.name().clone(),
                ])));
            }
        }
        errors
    }
}
