use tracing::error;

use hemtt_common::{error::thiserror, reporting::Code};

use crate::parse::Rule;

#[derive(thiserror::Error, Debug)]
/// Errors that can occur during preprocessing
pub enum Error {
    #[error("Coded error")]
    /// A coded error
    Code(Box<dyn Code>),
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(Box<std::io::Error>),
    #[error("Pest Error: {0}")]
    /// [`pest::error::Error`]
    Pest(Box<pest::error::Error<Rule>>),
    /// Workspace error
    #[error("Workspace Error: {0}")]
    Vfs(#[from] hemtt_common::workspace::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Self::Pest(Box::new(e))
    }
}

impl Error {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    /// Generate a user friendly report
    pub fn get_code(&self) -> Option<Box<&dyn Code>> {
        match self {
            Self::Code(c) => Some(Box::new(&**c)),
            Self::Io(_) | Self::Pest(_) | Self::Vfs(_) => None,
        }
    }
}
