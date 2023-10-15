use crate::BISignVersion;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Options {
    version: Option<BISignVersion>,

    #[serde(default)]
    authority: Option<String>,
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
}
