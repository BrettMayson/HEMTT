use std::sync::Arc;

use hemtt_common::project::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

use crate::Str;

use super::Analyze;

impl Analyze for Str {
    fn warnings(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }

    fn errors(&self, _: Option<&ProjectConfig>, _processed: &Processed) -> Vec<Arc<dyn Code>> {
        vec![]
    }
}
