use hemtt_common::reporting::Code;
use hemtt_preprocessor::Processed;

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
