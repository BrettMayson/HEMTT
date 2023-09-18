use hemtt_common::reporting::{Code, Processed};

use crate::{
    analyze::codes::{ce1_invalid_value::InvalidValue, ce2_invalid_value_macro::InvalidValueMacro},
    Value,
};

use super::Analyze;

impl Analyze for Value {
    fn valid(&self) -> bool {
        match self {
            Self::Str(s) => s.valid(),
            Self::Number(n) => n.valid(),
            Self::Array(a) => a.valid(),
            Self::UnexpectedArray(_) | Self::Invalid(_) => false,
        }
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Str(s) => s.warnings(processed),
            Self::Number(n) => n.warnings(processed),
            Self::Array(a) | Self::UnexpectedArray(a) => a.warnings(processed),
            Self::Invalid(_) => vec![],
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Str(s) => s.errors(processed),
            Self::Number(n) => n.errors(processed),
            Self::Array(a) | Self::UnexpectedArray(a) => a.errors(processed),
            Self::Invalid(invalid) => {
                // An unquoted string or otherwise invalid value
                vec![{
                    if processed
                        .mapping(invalid.start)
                        .is_some_and(hemtt_common::reporting::Mapping::was_macro)
                    {
                        Box::new(InvalidValueMacro::new(invalid.clone()))
                    } else {
                        Box::new(InvalidValue::new(invalid.clone()))
                    }
                }]
            }
        }
    }
}
