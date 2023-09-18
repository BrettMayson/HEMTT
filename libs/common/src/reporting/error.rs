use super::Code;

#[derive(thiserror::Error, Debug)]
/// Errors that can occur during preprocessing
pub enum Error {
    #[error("Coded error")]
    /// A coded error
    Code(Box<dyn Code>),
    /// [`hemtt_common::workspace::Error`]
    #[error("Workspace Error: {0}")]
    Workspace(#[from] crate::workspace::Error),
}

impl Error {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    /// Generate a user friendly report
    pub fn get_code(&self) -> Option<Box<&dyn Code>> {
        match self {
            Self::Code(c) => Some(Box::new(&**c)),
            Self::Workspace(_) => None,
        }
    }
}
