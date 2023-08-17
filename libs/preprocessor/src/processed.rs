use hemtt_common::workspace::WorkspacePath;

use crate::{output::Output, processor::Processor, Error};

#[derive(Debug, Default)]
/// A processed file
pub struct Processed {
    pub(crate) sources: Vec<WorkspacePath>,
    pub(crate) output: Vec<Output>,
}

impl Processed {
    /// Process a file from a workspace path
    pub fn new(path: &WorkspacePath) -> Result<Self, Error> {
        Processor::run(path)
    }

    #[must_use]
    /// Get the raw output from the preprocessor
    pub const fn output(&self) -> &Vec<Output> {
        &self.output
    }

    /// Get the output suitable for further processing
    /// Ignores certain tokens
    pub fn to_source(&self) -> String {
        self.output
            .iter()
            .map(Output::to_source)
            .collect::<String>()
    }
}

impl ToString for Processed {
    fn to_string(&self) -> String {
        self.output
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<String>()
    }
}
