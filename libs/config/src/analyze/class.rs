use hemtt_common::reporting::{Code, Processed};

use crate::Class;

use super::Analyze;

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
            Self::Local { properties, .. } => properties
                .iter()
                .flat_map(|p| p.errors(processed))
                .collect::<Vec<_>>(),
        }
    }
}
