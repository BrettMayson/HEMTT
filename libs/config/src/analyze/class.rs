use hemtt_common::reporting::{Code, Processed};
use hemtt_project::ProjectConfig;

use crate::Class;

use super::Analyze;

impl Analyze for Class {
    fn valid(&self, project: Option<&ProjectConfig>) -> bool {
        match self {
            Self::External { .. } => true,
            Self::Local { properties, .. } | Self::Root { properties, .. } => {
                properties.iter().all(|p| p.valid(project))
            }
        }
    }

    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Box<dyn Code>> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => properties
                .iter()
                .flat_map(|p| p.warnings(project, processed))
                .collect::<Vec<_>>(),
        }
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::External { .. } => vec![],
            Self::Local { properties, .. } | Self::Root { properties, .. } => properties
                .iter()
                .flat_map(|p| p.errors(project, processed))
                .collect::<Vec<_>>(),
        }
    }
}
