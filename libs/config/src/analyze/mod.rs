use hemtt_error::{processed::Processed, Code};

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
    fn valid(&self) -> bool;

    fn warnings(&self, processed: &Processed) -> Vec<Box<dyn Code>>;

    fn errors(&self, processed: &Processed) -> Vec<Box<dyn Code>>;
}
