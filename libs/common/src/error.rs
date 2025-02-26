//! HEMTT - Error Types

#![allow(missing_docs)]

#[derive(thiserror::Error, Debug)]
/// HEMTT Error
pub enum Error {
    #[error("Prefix error: {0}")]
    Prefix(#[from] crate::prefix::Error),

    #[error("Invalid config: {0}")]
    ConfigInvalid(String),

    #[error("Launch configuration extends non-existent configuration: {0} -> {1}")]
    LaunchConfigExtendsMissing(String, String),
    #[error("Launch configuration extends itself: {0}")]
    LaunchConfigExtendsSelf(String),
    #[error(
        "Launch configuration source conflict. They can exist in either `project.toml` or `launch.toml`, not both."
    )]
    LaunchConfigConflict,

    #[error(
        "Lints configuration source conflict. They can exist in either `project.toml` or `lints.toml`, not both."
    )]
    LintsConfigConflict,

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
