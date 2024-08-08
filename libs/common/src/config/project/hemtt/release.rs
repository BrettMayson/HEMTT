use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for `hemtt release`
pub struct ReleaseOptions {
    folder: String,
    sign: bool,
    archive: bool,
}

impl ReleaseOptions {
    /// Name to use for release archives
    /// Defaults to the project prefix
    pub fn folder(&self) -> &str {
        &self.folder
    }

    /// Should the PBOs be signed?
    /// Defaults to true
    pub const fn sign(&self) -> bool {
        self.sign
    }

    /// Create an archive of the release
    /// Defaults to true
    pub const fn archive(&self) -> bool {
        self.archive
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Release specific configuration
pub struct ReleaseOptionsFile {
    #[serde(default)]
    folder: Option<String>,

    #[serde(default)]
    sign: Option<bool>,

    #[serde(default)]
    archive: Option<bool>,
}

impl ReleaseOptionsFile {
    pub fn into_config(self, prefix: &str) -> ReleaseOptions {
        ReleaseOptions {
            folder: self.folder.unwrap_or_else(|| prefix.to_string()),
            sign: self.sign.unwrap_or(true),
            archive: self.archive.unwrap_or(true),
        }
    }
}
