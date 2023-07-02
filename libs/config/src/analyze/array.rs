use hemtt_error::{processed::Processed, Code};

use crate::{Array, Item};

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
            Self::Invalid(_) => vec![],
        }
    }
}
