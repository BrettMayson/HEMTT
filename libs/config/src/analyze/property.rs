use std::ops::Range;

use hemtt_common::reporting::{Code, Processed};
use hemtt_project::ProjectConfig;

use super::{
    codes::{
        ce4_missing_semicolon::MissingSemicolon, ce5_unexpected_array::UnexpectedArray,
        ce6_expected_array::ExpectedArray,
    },
    Analyze,
};
use crate::{Property, Value};

impl Analyze for Property {
    fn valid(&self, project: Option<&ProjectConfig>) -> bool {
        match self {
            Self::Entry { value, .. } => value.valid(project),
            Self::Class(c) => c.valid(project),
            Self::Delete(_) => true,
            Self::MissingSemicolon(_, _) => false,
        }
    }

    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Box<dyn Code>> {
        match self {
            Self::Entry { value, .. } => value.warnings(project, processed),
            Self::Class(c) => c.warnings(project, processed),
            Self::Delete(_) | Self::MissingSemicolon(_, _) => vec![],
        }
    }

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Box<dyn Code>> {
        match self {
            Self::Entry { value, .. } => {
                let mut errors = value.errors(project, processed);
                errors.extend(unexpected_array(self));
                errors.extend(expected_array(self));
                errors
            }
            Self::Class(c) => c.errors(project, processed),
            Self::Delete(_) => vec![],
            Self::MissingSemicolon(_, span) => vec![missing_semicolon(span)],
        }
    }
}

fn missing_semicolon(span: &Range<usize>) -> Box<dyn Code> {
    Box::new(MissingSemicolon::new(span.clone()))
}

fn unexpected_array(property: &Property) -> Vec<Box<dyn Code>> {
    let Property::Entry {
        value: Value::UnexpectedArray(_),
        ..
    } = property
    else {
        return vec![];
    };
    vec![Box::new(UnexpectedArray::new(property.clone()))]
}

fn expected_array(property: &Property) -> Vec<Box<dyn Code>> {
    let Property::Entry {
        value,
        expected_array,
        ..
    } = property
    else {
        return vec![];
    };
    if !expected_array {
        return vec![];
    }
    if let Value::Array(_) = value {
        return vec![];
    }
    // If we can't tell what the value is, we can't tell if it's an array or not
    if let Value::Invalid(_) = value {
        return vec![];
    }
    vec![Box::new(ExpectedArray::new(property.clone()))]
}
