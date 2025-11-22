use crate::analyze::inspector::{VarSource, game_value::GameValue};
use std::{hash::Hash, ops::Range, vec};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Issue {
    InvalidArgs {
        command: String,
        span: Range<usize>,
        variant: InvalidArgs,
    },
    Undefined(String, Range<usize>, bool),
    Unused(String, VarSource),
    Shadowed(String, Range<usize>),
    NotPrivate(String, Range<usize>),
    CountArrayComparison(bool, Range<usize>, String),
    InvalidReturnType {
        variant: InvalidArgs,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InvalidArgs {
    TypeNotExpected {
        expected: Vec<GameValue>,
        found: Vec<GameValue>,
        span: Range<usize>,
    },
    DefaultDifferentType {
        expected: Vec<GameValue>,
        found: Vec<GameValue>,
        span: Range<usize>,
        default: Option<Range<usize>>,
    },
    NilResultUsed {
        found: Vec<GameValue>,
        span: Range<usize>,
    },
    ExpectedDifferentTypeHeader {
        expected: Vec<GameValue>,
        found: Vec<GameValue>,
        span: Range<usize>,
    },
    InvalidReturnType {
        expected: Vec<GameValue>,
        found: Vec<GameValue>,
        span: Range<usize>,
    },
}

impl InvalidArgs {
    #[must_use]
    pub fn note(&self) -> String {
        let found = self
            .found_types()
            .iter()
            .map(GameValue::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "found type{} was {found}",
            if self.found_types().len() > 1 {
                "s"
            } else {
                ""
            }
        )
    }

    #[must_use]
    pub fn message(&self, command: &str) -> String {
        match self {
            Self::TypeNotExpected { .. } => format!("Invalid argument type for `{command}`"),
            Self::NilResultUsed { .. } => format!("Invalid argument (nil) for `{command}`"),
            Self::DefaultDifferentType { default, .. } => {
                if default.is_none() {
                    String::from(
                        "Default value is not an expected type for the parameter (from header)",
                    )
                } else {
                    String::from("Default value is not an expected type for the parameter")
                }
            }
            Self::ExpectedDifferentTypeHeader { .. } => {
                String::from("Expected value does not match (from Header)")
            }
            Self::InvalidReturnType { .. } => {
                String::from("Invalid function return type (from Header)")
            }
        }
    }

    #[must_use]
    pub fn label_message(&self) -> String {
        match self {
            Self::NilResultUsed { .. } => String::from("expected non-nil value"),
            Self::TypeNotExpected { .. }
            | Self::DefaultDifferentType { .. }
            | Self::ExpectedDifferentTypeHeader { .. }
            | Self::InvalidReturnType { .. } => {
                format!(
                    "expected {}",
                    self.expected_types()
                        .iter()
                        .map(GameValue::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }

    #[must_use]
    pub fn found_types(&self) -> Vec<GameValue> {
        match self {
            Self::TypeNotExpected { found, .. }
            | Self::DefaultDifferentType { found, .. }
            | Self::ExpectedDifferentTypeHeader { found, .. }
            | Self::NilResultUsed { found, .. }
            | Self::InvalidReturnType { found, .. } => found.clone(),
        }
    }

    #[must_use]
    pub fn expected_types(&self) -> Vec<GameValue> {
        match self {
            Self::NilResultUsed { .. } => vec![],
            Self::TypeNotExpected { expected, .. }
            | Self::DefaultDifferentType { expected, .. }
            | Self::ExpectedDifferentTypeHeader { expected, .. }
            | Self::InvalidReturnType { expected, .. } => expected.clone(),
        }
    }

    #[must_use]
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::TypeNotExpected { span, .. }
            | Self::DefaultDifferentType { span, .. }
            | Self::ExpectedDifferentTypeHeader { span, .. }
            | Self::InvalidReturnType { span, .. }
            | Self::NilResultUsed { span, .. } => span.clone(),
        }
    }
}
