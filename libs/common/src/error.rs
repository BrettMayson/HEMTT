//! HEMTT - Error Types

#![allow(missing_docs)]

pub use thiserror;

#[derive(thiserror::Error, Debug)]
/// HEMTT Error
pub enum Error {
    #[error("Invalid config: {0}")]
    /// Invalid config
    ConfigInvalid(String),

    #[error("Prefix error: {0}")]
    Prefix(#[from] crate::prefix::Error),

    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml Error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Version Error: {0}")]
    Version(#[from] crate::version::Error),
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}
