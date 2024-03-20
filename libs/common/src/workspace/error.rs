#[derive(thiserror::Error, Debug)]
/// Errors that can occur while parsing a version
pub enum Error {
    #[error("VfsError: {0}")]
    /// [`vfs::VfsError`]
    Vfs(Box<vfs::VfsError>),

    #[error("PrefixError: {0}")]
    /// [`crate::prefix::Error`]
    Prefix(#[from] crate::prefix::Error),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}
