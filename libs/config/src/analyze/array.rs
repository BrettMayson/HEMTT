use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_common::reporting::{Code, Processed};

use crate::{
    analyze::codes::{ce1_invalid_value::InvalidValue, ce2_invalid_value_macro::InvalidValueMacro},
    Array, Item,
};

use super::Analyze;

impl Analyze for Array {
    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Arc<dyn Code>> {
        self.items
            .iter()
            .flat_map(|i| i.warnings(project, processed))
            .collect::<Vec<_>>()
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>> {
        self.items
            .iter()
            .flat_map(|i| i.errors(project, processed))
            .collect::<Vec<_>>()
    }
}

impl Analyze for Item {
    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Arc<dyn Code>> {
        match self {
            Self::Str(s) => s.warnings(project, processed),
            Self::Number(n) => n.warnings(project, processed),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.warnings(project, processed))
                .collect::<Vec<_>>(),
            Self::Invalid(_) => vec![],
        }
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>> {
        match self {
            Self::Str(s) => s.errors(project, processed),
            Self::Number(n) => n.errors(project, processed),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.errors(project, processed))
                .collect::<Vec<_>>(),
            Self::Invalid(invalid) =>
            // An unquoted string or otherwise invalid value
            {
                vec![{
                    if processed
                        .mapping(invalid.start)
                        .is_some_and(hemtt_common::reporting::Mapping::was_macro)
                    {
                        Arc::new(InvalidValueMacro::new(invalid.clone(), processed))
                    } else {
                        Arc::new(InvalidValue::new(invalid.clone(), processed))
                    }
                }]
            }
        }
    }
}
