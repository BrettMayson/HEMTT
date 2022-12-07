use hemtt_error::{thiserror, PrettyError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid addon location: {0}")]
    InvalidAddonLocation(String),
    #[error("Unable to create link: {0}")]
    Link(String),

    #[error("Project error: {0}")]
    Project(#[from] hemtt_bin_project::Error),
    #[error("Preprocessor error: {0}")]
    Preprocessor(#[from] hemtt_preprocessor::Error),
    #[error("Config error: {0}")]
    Config(#[from] hemtt_config::Error),
    #[error("PBO error: {0}")]
    Pbo(#[from] hemtt_pbo::Error),
    #[error("Signing error: {0}")]
    Signing(#[from] hemtt_signing::Error),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
    #[error("Glob Error: {0}")]
    GlobPattern(#[from] glob::PatternError),
    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}

impl PrettyError for Error {
    fn brief(&self) -> String {
        self.to_string()
    }

    fn details(&self) -> Option<String> {
        None
    }

    fn help(&self) -> Option<String> {
        match self {
            Self::Preprocessor(e) => e.help(),
            Self::Config(e) => e.help(),
            _ => None,
        }
    }

    fn source(&self) -> Option<Box<hemtt_error::Source>> {
        match self {
            Self::Preprocessor(e) => e.source(),
            Self::Config(e) => e.source(),
            // Self::Pbo(e) => e.source(),
            _ => None,
        }
    }

    fn trace(&self) -> Vec<hemtt_error::Source> {
        match self {
            Self::Preprocessor(e) => e.trace(),
            Self::Config(e) => e.trace(),
            _ => vec![],
        }
    }
}
