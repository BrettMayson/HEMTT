use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use hemtt_error::{thiserror, Code};
use tracing::error;

use crate::{parse::Rule, Token};

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
    #[error("Vfs Error: {0}")]
    /// [`vfs::Error`]
    Vfs(Box<vfs::error::VfsError>),
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

impl From<vfs::error::VfsError> for Error {
    fn from(e: vfs::error::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}

impl Error {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    /// Generate a user friendly report
    pub fn get_code(&self) -> Option<Box<&dyn Code>> {
        match self {
            Self::Code(c) => Some(Box::new(&**c)),
            Self::Io(_) => todo!(),
            Self::Pest(_) => todo!(),
            Self::Vfs(_) => todo!(),
        }
    }
}
