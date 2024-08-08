use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

use crate::Config;

use super::Analyze;

mod duplicate_classes;
mod duplicate_properties;
mod external_missing;
mod external_parent_case;
mod magwells;

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
        warnings.extend(external_parent_case::warn(&self.0, processed));
        if let Some(project) = project {
            warnings.extend(magwells::missing_magazine(project, self, processed));
        }
        warnings
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>> {
        let mut errors = self
            .0
            .iter()
            .flat_map(|p| p.errors(project, processed))
            .collect::<Vec<_>>();
        errors.extend(duplicate_classes::error(&self.0, processed));
        errors.extend(external_missing::error(&self.0, processed));
        errors.extend(duplicate_properties::duplicate_properties(
            &self.0, processed,
        ));
        errors
    }
}
