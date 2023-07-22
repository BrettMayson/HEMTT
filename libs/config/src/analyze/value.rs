use hemtt_error::{processed::Processed, tokens::Symbol, Code};

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
                    let map = processed.original_col(invalid.start).unwrap();
                    let mut root = map.token();
                    let mut at_root = true;
                    while let Some(parent) = root.parent() {
                        if parent.symbol() == &Symbol::Word("include".to_owned()) {
                            break;
                        }
                        root = parent;
                        at_root = false;
                    }
                    if at_root {
                        Box::new(InvalidValue::new(invalid.clone()))
                    } else {
                        Box::new(InvalidValueMacro::new(invalid.clone()))
                    }
                }]
            }
        }
    }
}
