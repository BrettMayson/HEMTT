use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

use crate::{
    analyze::codes::{ce1_invalid_value::InvalidValue, ce2_invalid_value_macro::InvalidValueMacro},
    Value,
};

use super::Analyze;

impl Analyze for Value {
    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Arc<dyn Code>> {
        match self {
            Self::Str(s) => s.warnings(project, processed),
            Self::Number(n) => n.warnings(project, processed),
            Self::Expression(e) => e.warnings(project, processed),
            Self::Array(a) | Self::UnexpectedArray(a) => a.warnings(project, processed),
            Self::Invalid(_) => vec![],
        }
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>> {
        match self {
            Self::Str(s) => s.errors(project, processed),
            Self::Number(n) => n.errors(project, processed),
            Self::Expression(e) => e.errors(project, processed),
            Self::Array(a) | Self::UnexpectedArray(a) => a.errors(project, processed),
            Self::Invalid(invalid) => {
                // An unquoted string or otherwise invalid value
                vec![{
                    if processed
                        .mapping(invalid.start)
                        .is_some_and(hemtt_workspace::reporting::Mapping::was_macro)
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
