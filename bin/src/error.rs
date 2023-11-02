use hemtt_common::error::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("`.hemtt/project.toml` not found")]
    ConfigNotFound,
    #[error("Launch config not found: {0}")]
    LaunchConfigNotFound(String),

    #[error("ASC: {0}")]
    #[cfg(not(target_os = "macos"))]
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

    #[error("One or more files failed linting")]
    LintFailed,

    #[error("Unable to create link: {0}")]
    #[allow(dead_code)] // Unused on Linux and Mac
    Link(String),
    #[error("Arma 3 not found in Steam")]
    Arma3NotFound,
    #[error("Workshop folder not found")]
    WorkshopNotFound,
    #[error("Workshop mod not found: {0}")]
    WorkshopModNotFound(String),
    #[error("Preset not found: {0}")]
    PresetNotFound(String),

    #[error("Main prefix not found: {0}")]
    MainPrefixNotFound(String),

    #[error("Preprocessor error: {0}")]
    Preprocessor(#[from] hemtt_preprocessor::Error),
    #[error("Config error: {0}")]
    Config(#[from] hemtt_config::Error),
    #[error("PBO error: {0}")]
    Pbo(#[from] hemtt_pbo::Error),
    #[error("Prefix error: {0}")]
    Prefix(#[from] hemtt_common::prefix::Error),
    #[error("`a hemtt project file is invalid: {0}")]
    Project(#[from] hemtt_common::project::Error),
    #[error("Signing error: {0}")]
    Signing(#[from] hemtt_signing::Error),
    #[error("Version Error: {0}")]
    Version(#[from] hemtt_common::version::Error),
    #[error("Workspace Error: {0}")]
    Workspace(#[from] hemtt_common::workspace::Error),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Dialoguer Error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
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
    #[error("Vfs Error {0}")]
    Vfs(Box<vfs::VfsError>),
    #[error("Walkdir Error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Rhai Parse Error: {0}")]
    RhaiParse(#[from] rhai::ParseError),
    #[error("Rhai Script Error: {0}")]
    /// because of annyoing send + sync I don't care about
    RhaiScript(String),
}

impl From<vfs::VfsError> for Error {
    fn from(e: vfs::VfsError) -> Self {
        Self::Vfs(Box::new(e))
    }
}

impl From<Box<rhai::EvalAltResult>> for Error {
    fn from(e: Box<rhai::EvalAltResult>) -> Self {
        Self::RhaiScript(e.to_string())
    }
}
