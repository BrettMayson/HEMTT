use serde::{Deserialize, Serialize};

use crate::BISignVersion;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningConfig {
    version: BISignVersion,

    authority: Option<String>,
}

impl SigningConfig {
    pub const fn version(&self) -> BISignVersion {
        self.version
    }

    pub const fn authority(&self) -> Option<&String> {
        self.authority.as_ref()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SigningSectionFile {
    version: Option<BISignVersion>,

    #[serde(default)]
    authority: Option<String>,
}

impl From<SigningSectionFile> for SigningConfig {
    fn from(file: SigningSectionFile) -> Self {
        Self {
            version: file.version.unwrap_or_default(),
            authority: file.authority,
        }
    }
}
