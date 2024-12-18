use std::sync::Arc;

use tracing::error;

use hemtt_workspace::reporting::Code;

#[derive(thiserror::Error, Debug)]
/// Errors that can occur during preprocessing
pub enum Error {
    #[error("Coded error: {0:?}")]
    /// A coded error
    Code(Arc<dyn Code>),
    #[error("IO Error: {0}")]
    /// [`std::io::Error`]
    Io(Box<std::io::Error>),
    /// [`hemtt_workspace::Error`]
    #[error("Workspace Error: {0}")]
    Workspace(#[from] hemtt_workspace::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl Error {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    /// Generate a user friendly report
    pub fn get_code(&self) -> Option<Box<&dyn Code>> {
        match self {
            Self::Code(c) => Some(Box::new(&**c)),
            Self::Io(_) | Self::Workspace(_) => None,
        }
    }
}
