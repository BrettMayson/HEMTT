use hemtt_common::error::{processed::Processed, tokens::Symbol, Code};

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
