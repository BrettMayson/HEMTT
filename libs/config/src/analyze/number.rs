use crate::Number;

use super::Analyze;

impl Analyze for Number {
    fn valid(&self) -> bool {
        true
    }

    fn warnings(&self, _processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        vec![]
    }

    fn errors(&self, _processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        vec![]
    }
}
