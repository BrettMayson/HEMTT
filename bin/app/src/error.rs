use hemtt_error::{thiserror, PrettyError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid addon location: {0}")]
    InvalidAddonLocation(String),

    #[error("Project error: {0}")]
    Project(#[from] hemtt_bin_project::Error),
    #[error("Preprocessor error: {0}")]
    Preprocessor(#[from] hemtt_preprocessor::Error),
    #[error("Config error: {0}")]
    Config(#[from] hemtt_config::Error),
    #[error("PBO error: {0}")]
    Pbo(#[from] hemtt_pbo::Error),

    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    // #[error("Vfs Error")]
    // Vfs(#[from] vfs::VfsError),
}

impl PrettyError for Error {
    fn brief(&self) -> String {
        self.to_string()
    }

    fn details(&self) -> Option<String> {
        None
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn source(&self) -> Option<Box<hemtt_error::Source>> {
        None
    }
}
