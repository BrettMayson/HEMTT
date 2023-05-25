use hemtt_error::{thiserror, PrettyError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("`.hemtt/project.toml` not found")]
    ConfigNotFound,
    #[error("Invalid config: {0}")]
    ConfigInvalid(String),
    #[error("Launch config not found: {0}")]
    LaunchConfigNotFound(String),

    #[error("ASC: {0}")]
    ArmaScriptCompiler(String),

    #[error("Folder already exists: {0}")]
    NewFolderExists(String),
    #[error("New can only be ran in an interactive terminal")]
    NewNoInput,

    #[error("Invalid addon location: {0}")]
    AddonLocationInvalid(String),
    #[error("Optional addon not found: {0}")]
    AddonOptionalNotFound(String),
    #[error("Addon prefix not found: {0}")]
    AddonPrefixMissing(String),

    #[error("Hook signaled failure: {0}")]
    HookFatal(String),
    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    #[error("Unable to create link: {0}")]
    #[allow(dead_code)] // Unused on Linux and Mac
    Link(String),
    #[error("Arma 3 not found in Steam")]
    Arma3NotFound,
    #[error("Workshop folder not found")]
    WorkshopNotFound,
    #[error("Workshop mod not found: {0}")]
    WorkshopModNotFound(String),
    #[error("Main prefix not found: {0}")]
    MainPrefixNotFound(String),

    #[error("Preprocessor error: {0}")]
    Preprocessor(#[from] hemtt_preprocessor::Error),
    #[error("Config error: {0}")]
    Config(#[from] hemtt_config::Error),
    #[error("PBO error: {0}")]
    Pbo(#[from] hemtt_pbo::Error),
    #[error("Signing error: {0}")]
    Signing(#[from] hemtt_signing::Error),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Git Error: {0}")]
    Git(#[from] git2::Error),
    #[error("Glob Error: {0}")]
    GlobError(#[from] glob::GlobError),
    #[error("Glob Pattern Error: {0}")]
    GlobPattern(#[from] glob::PatternError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Toml Error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Version Error: {0}")]
    Version(#[from] hemtt_version::Error),
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
    #[error("Walkdir Error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Rhai Error: {0}")]
    Rhai(#[from] rhai::ParseError),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}

impl PrettyError for Error {
    fn brief(&self) -> String {
        match self {
            Self::Preprocessor(e) => e.brief(),
            Self::Config(e) => e.brief(),
            _ => self.to_string(),
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::Preprocessor(e) => e.details(),
            Self::Config(e) => e.details(),
            _ => None,
        }
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
