use hemtt_common::error::{processed::Processed, Code};

use crate::Str;

use super::Analyze;

impl Analyze for Str {
    fn valid(&self) -> bool {
        true
    }

    fn warnings(&self, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }

    fn errors(&self, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }
}
