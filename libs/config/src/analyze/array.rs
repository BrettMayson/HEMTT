use hemtt_common::reporting::{Code, Processed};

use crate::{
    analyze::codes::{ce1_invalid_value::InvalidValue, ce2_invalid_value_macro::InvalidValueMacro},
    Array, Item,
};

use super::Analyze;

impl Analyze for Array {
    fn valid(&self) -> bool {
        self.items.iter().all(Analyze::valid)
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        self.items
            .iter()
            .flat_map(|i| i.warnings(processed))
            .collect::<Vec<_>>()
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        self.items
            .iter()
            .flat_map(|i| i.errors(processed))
            .collect::<Vec<_>>()
    }
}

impl Analyze for Item {
    fn valid(&self) -> bool {
        match self {
            Self::Str(s) => s.valid(),
            Self::Number(n) => n.valid(),
            Self::Array(a) => a.iter().all(Analyze::valid),
            Self::Invalid(_) => false,
        }
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Str(s) => s.warnings(processed),
            Self::Number(n) => n.warnings(processed),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.warnings(processed))
                .collect::<Vec<_>>(),
            Self::Invalid(_) => vec![],
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Str(s) => s.errors(processed),
            Self::Number(n) => n.errors(processed),
            Self::Array(a) => a
                .iter()
                .flat_map(|i| i.errors(processed))
                .collect::<Vec<_>>(),
            Self::Invalid(invalid) =>
            // An unquoted string or otherwise invalid value
            {
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
