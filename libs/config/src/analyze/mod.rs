use hemtt_common::reporting::{Code, Processed};
use hemtt_project::ProjectConfig;

mod array;
mod class;
pub mod codes;
mod config;
mod number;
mod property;
mod str;
mod value;

/// Trait for rapifying objects
pub trait Analyze {
    /// Check if the object is valid and can be rapified
    fn valid(&self, project: Option<&ProjectConfig>) -> bool;

    fn warnings(
        &self,
        project: Option<&ProjectConfig>,
        processed: &Processed,
    ) -> Vec<Box<dyn Code>>;

    fn errors(&self, project: Option<&ProjectConfig>, processed: &Processed) -> Vec<Box<dyn Code>>;
}
