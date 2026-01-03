#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Config error: {0}")]
    Config(String),

    #[error("`.hemtt/project.toml` not found")]
    ConfigNotFound,

    #[error("Not in a git repository: {0}")]
    NotInGitRepository(String),

    #[error("Unable to create link: {0}")]
    #[allow(dead_code)] // Unused on Linux and Mac
    Link(String),

    #[error("Project error: {0}")]
    Common(#[from] hemtt_common::error::Error),
    #[error("Preprocessor error: {0}")]
    Preprocessor(#[from] hemtt_preprocessor::Error),
    #[error("PBO error: {0}")]
    Pbo(#[from] hemtt_pbo::Error),
    #[error("Prefix error: {0}")]
    Prefix(#[from] hemtt_common::prefix::Error),
    #[error("Signing error: {0}")]
    Signing(#[from] hemtt_signing::Error),
    #[error("Version Error: {0}")]
    Version(#[from] hemtt_common::version::Error),
    #[error("Workspace Error: {0}")]
    Workspace(#[from] hemtt_workspace::Error),
    #[error("Wss Error: {0}")]
    Wss(#[from] hemtt_wss::Error),
    #[error("Sqf Error: {0}")]
    Sqf(#[from] hemtt_sqf::Error),
    #[error("Addon Error: {0}")]
    Addon(#[from] hemtt_workspace::addons::Error),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Invalid value: {0}")]
    InvalidValueName(String),

    #[error("Dialoguer Error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("Glob Error: {0}")]
    GlobError(#[from] glob::GlobError),
    #[error("Glob Pattern Error: {0}")]
    GlobPattern(#[from] glob::PatternError),
    #[error("Image Error: {0}")]
    Image(#[from] image::ImageError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
    #[error("Walkdir Error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}
