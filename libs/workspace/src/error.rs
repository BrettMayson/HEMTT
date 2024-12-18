#[derive(Debug, thiserror::Error)]
/// Errors that can occur while parsing a version
pub enum Error {
    #[error("Addon error: {0}")]
    Addon(#[from] crate::addons::Error),

    #[error("Project error: {0}")]
    Common(#[from] hemtt_common::error::Error),

    #[error("Prefix error: {0}")]
    Prefix(#[from] hemtt_common::prefix::Error),

    #[error("VfsError: {0}")]
    /// [`vfs::VfsError`]
    Vfs(Box<vfs::VfsError>),

    #[error("IOError: {0}")]
    /// [`std::io::Error`]
    Io(#[from] std::io::Error),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}
