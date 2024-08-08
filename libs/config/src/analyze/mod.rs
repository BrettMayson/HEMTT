use std::sync::Arc;

use hemtt_common::config::ProjectConfig;
use hemtt_workspace::reporting::{Code, Processed};

mod array;
mod class;
pub mod codes;
mod config;
mod expression;
mod number;
mod property;
mod str;
mod value;

mod model;

pub use model::CfgPatch;

/// Trait for rapifying objects
pub trait Analyze {
    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Arc<dyn Code>>;

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Arc<dyn Code>>;
}
