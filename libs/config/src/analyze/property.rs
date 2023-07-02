use std::ops::Range;

use hemtt_error::{processed::Processed, Code};

use super::{
    codes::{
        ce4_missing_semicolon::MissingSemicolon, ce5_unexpected_array::UnexpectedArray,
        ce6_expected_array::ExpectedArray,
    },
    Analyze,
};
use crate::{Property, Value};

impl Analyze for Property {
    fn valid(&self) -> bool {
        !matches!(
            self,
            Self::Entry {
                value: Value::Invalid(_),
                ..
            } | Self::MissingSemicolon(_, _)
        )
    }

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Entry { value, .. } => value.warnings(processed),
            Self::Class(c) => c.warnings(processed),
            Self::Delete(_) | Self::MissingSemicolon(_, _) => vec![],
        }
    }

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Entry { value, .. } => {
                let mut errors = value.errors(processed);
                errors.extend(unexpected_array(self));
                errors.extend(expected_array(self));
                errors
            }
            Self::Class(c) => c.errors(processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, span) => vec![missing_semicolon(span)],
        }
    }
}

fn missing_semicolon(span: &Range<usize>) -> Box<dyn Code> {
    Box::new(MissingSemicolon::new(span.clone()))
}

fn unexpected_array(property: &Property) -> Vec<Box<dyn Code>> {
    let Property::Entry { value: Value::UnexpectedArray(_), .. } = property else {
        return vec![];
    };
    vec![Box::new(UnexpectedArray::new(property.clone()))]
}

fn expected_array(property: &Property) -> Vec<Box<dyn Code>> {
    let Property::Entry { value, expected_array, .. } = property else {
        return vec![];
    };
    if !expected_array {
        return vec![];
    }
    if let Value::Array(_) = value {
        return vec![];
    }
    vec![Box::new(ExpectedArray::new(property.clone()))]
}
