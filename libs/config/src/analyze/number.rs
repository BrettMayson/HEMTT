use hemtt_common::reporting::{Code, Processed};
use hemtt_project::ProjectConfig;

use crate::Number;

use super::Analyze;

impl Analyze for Number {
    fn valid(&self, _: Option<&ProjectConfig>) -> bool {
        true
    }

    fn warnings(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }

    fn errors(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Box<dyn Code>> {
        vec![]
    }
}
