use std::path::PathBuf;

use crate::templates::Templates;

#[derive(Debug)]
pub struct IOPathError {
    pub source: std::io::Error,
    pub path: std::path::PathBuf,
    // pub message: Option<String>,
}

#[derive(Debug)]
pub struct PreprocessError {
    pub path: Option<String>,
    pub message: String,
    pub source: Box<HEMTTError>,
}

#[derive(Debug)]
pub enum HEMTTError {
    User(String),
    UserHint(String, String),
    Generic(String),
    Preprocess(PreprocessError),
    IO(std::io::Error),
    IOPath(IOPathError),
    SemVer(semver::SemVerError),
    Vfs(vfs::VfsError),
    Handlebars(handlebars::RenderError),

    // Addon
    AddonConflict(String, crate::AddonLocation, crate::AddonLocation),
    AddonInvalidName(String),
    AddonInvalidLocation(String),

    // Project
    NoProjectFound,

    // Templates
    TemplateUnknown(String),

    // Release
    ReleaseExists(PathBuf),
}

impl HEMTTError {
    pub const fn can_submit_bug(&self) -> bool {
        !matches!(
            *self,
            Self::User(_)
                | Self::UserHint(_, _)
                | Self::AddonConflict(_, _, _)
                | Self::AddonInvalidName(_)
                | Self::AddonInvalidLocation(_)
                | Self::NoProjectFound
                | Self::TemplateUnknown(_)
                | Self::ReleaseExists(_)
        )
    }
}

#[macro_export]
macro_rules! aerror {
    ($e:expr) => {
        HEMTTError::Generic($e.to_string())
    };
    ($e:expr, $($p:expr),*) => {
        aerror!(format!($e, $($p,)*))
    };
}

impl std::fmt::Display for HEMTTError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::User(ref s) => write!(f, "{}", s),
            Self::UserHint(ref s, ref h) => write!(f, "{}\ntry: {}", s, h),
            Self::Generic(ref s) => write!(f, "{}", s),
            Self::Preprocess(ref e) => write!(f, "Preprocessor: {}", e.message),
            Self::IO(ref e) => write!(f, "IO error: {}", e),
            Self::IOPath(ref e) => write!(f, "IO error: `{:#?}`\n{}", e.path, e.source),
            Self::SemVer(ref e) => write!(f, "SemVer error: `{}`", e),
            Self::Vfs(ref e) => write!(f, "Vfs error: `{}`", e),
            Self::Handlebars(ref e) => write!(f, "Handlebars error: `{}`", e),

            // Addon
            Self::AddonConflict(ref name, ref target, ref other) => write!(
                f,
                "Addon conflict. `{}` cannot exist in `{}`, it exists in `{}`",
                name, target, other
            ),
            Self::AddonInvalidName(ref addon) => {
                write!(f, "Invalid characters in addon name: `{}`", addon)
            }
            Self::AddonInvalidLocation(ref loc) => write!(
                f,
                "Invalid addon location `{}`, {}",
                loc,
                crate::AddonLocation::options()
            ),

            // Project
            Self::NoProjectFound => write!(f, "No HEMTT Project found"),

            // Template
            Self::TemplateUnknown(ref template) => write!(
                f,
                "Unknown template: {}, {}",
                template,
                Templates::options()
            ),

            // Release
            Self::ReleaseExists(ref rel) => {
                write!(f, "Release already exists: `{}`", rel.display())
            }
        }
    }
}

impl std::error::Error for HEMTTError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Self::User(_) => Some(self),
            Self::UserHint(_, ref _h) => Some(self),
            Self::Generic(_) => Some(self),
            Self::Preprocess(ref e) => Some(&e.source),
            Self::IO(ref e) => Some(e),
            Self::IOPath(ref e) => Some(&e.source),
            Self::SemVer(ref e) => Some(e),
            Self::Vfs(ref e) => Some(e),
            Self::Handlebars(ref e) => Some(e),

            // Addon
            Self::AddonConflict(_, _, _) => Some(self),
            Self::AddonInvalidName(_) => Some(self),
            Self::AddonInvalidLocation(_) => Some(self),

            // Project
            Self::NoProjectFound => Some(self),

            // Template
            Self::TemplateUnknown(_) => Some(self),

            // Release
            Self::ReleaseExists(_) => Some(self),
        }
    }
}

impl From<std::io::Error> for HEMTTError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<semver::SemVerError> for HEMTTError {
    fn from(err: semver::SemVerError) -> Self {
        Self::SemVer(err)
    }
}

impl From<vfs::VfsError> for HEMTTError {
    fn from(err: vfs::VfsError) -> Self {
        Self::Vfs(err)
    }
}

impl From<String> for HEMTTError {
    fn from(err: String) -> Self {
        Self::Generic(err)
    }
}

impl From<handlebars::RenderError> for HEMTTError {
    fn from(err: handlebars::RenderError) -> Self {
        Self::Handlebars(err)
    }
}
