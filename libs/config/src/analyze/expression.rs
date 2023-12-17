use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_common::reporting::{Code, Processed};

use crate::Expression;

use super::Analyze;

impl Analyze for Expression {
    fn warnings(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }

    fn errors(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }
}
