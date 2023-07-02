use crate::Str;

use super::Analyze;

impl Analyze for Str {
    fn valid(&self) -> bool {
        true
    }

    fn warnings(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        vec![]
    }

    fn errors(&self, processed: &hemtt_preprocessor::Processed) -> Vec<String> {
        vec![]
    }
}
