use hemtt_common::error::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid config: {0}")]
    ConfigInvalid(String),

    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml Error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Version Error: {0}")]
    Version(#[from] hemtt_common::version::Error),
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}
