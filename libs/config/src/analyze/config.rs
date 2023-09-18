use std::collections::HashSet;

use hemtt_common::reporting::{Code, Processed};

use crate::{Class, Config, Property};

use super::{codes::ce7_missing_parent::MissingParrent, Analyze};

impl Analyze for Config {
    fn valid(&self) -> bool {
        self.0.iter().all(Analyze::valid)
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        self.0
            .iter()
            .flat_map(|p| p.warnings(processed))
            .collect::<Vec<_>>()
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        let mut errors = self
            .0
            .iter()
            .flat_map(|p| p.errors(processed))
            .collect::<Vec<_>>();
        let mut defined = HashSet::new();
        errors.extend(external_missing(&self.0, &mut defined));
        errors
    }
}

fn external_missing(properties: &[Property], defined: &mut HashSet<String>) -> Vec<Box<dyn Code>> {
    let mut errors: Vec<Box<dyn Code>> = Vec::new();
    for property in properties {
        if let Property::Class(c) = property {
            match c {
                Class::External { name } => {
                    if !defined.contains(&name.value) {
                        defined.insert(name.value.clone());
                    }
                }
                Class::Local {
                    parent, properties, ..
                } => {
                    if let Some(parent) = parent {
                        if !defined.contains(&parent.value) {
                            errors.push(Box::new(MissingParrent::new(c.clone())));
                        }
                    }
                    defined.insert(c.name().value.clone());
                    errors.extend(external_missing(properties, defined));
                }
            }
        }
    }
    errors
}
