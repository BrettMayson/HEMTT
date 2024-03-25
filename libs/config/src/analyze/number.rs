use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

use crate::Number;

use super::Analyze;

impl Analyze for Number {
    fn warnings(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }

    fn errors(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }
}
