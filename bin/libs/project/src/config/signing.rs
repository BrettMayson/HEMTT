use hemtt_pbo::BISignVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    version: Option<BISignVersion>,

    #[serde(default)]
    authority: Option<String>,

    #[serde(default)]
    include_git_hash: Option<bool>,
}

impl Options {
    #[must_use]
    pub const fn version(&self) -> BISignVersion {
        if let Some(version) = self.version {
            version
        } else {
            BISignVersion::V3
        }
    }

    /// Get the authority for signing
    ///
    /// # Errors
    /// Returns an error if the authority is not set
    pub const fn authority(&self) -> Option<&String> {
        self.authority.as_ref()
    }

    #[must_use]
    pub const fn include_git_hash(&self) -> bool {
        if let Some(include) = self.include_git_hash {
            include
        } else {
            false
        }
    }
}
