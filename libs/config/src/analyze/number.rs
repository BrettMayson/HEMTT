use hemtt_common::reporting::{Code, Processed};

use crate::Number;

use super::Analyze;

impl Analyze for Number {
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
