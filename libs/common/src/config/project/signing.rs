use serde::{Deserialize, Serialize};

use crate::BISignVersion;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningConfig {
    version: BISignVersion,

    authority: Option<String>,

    private_key_hash: Option<String>,
}

impl SigningConfig {
    pub const fn version(&self) -> BISignVersion {
        self.version
    }

    pub fn authority(&self) -> Option<&str> {
        self.authority.as_deref()
    }

    pub fn private_key_hash(&self) -> Option<&str> {
        self.private_key_hash.as_deref()
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SigningSectionFile {
    version: Option<BISignVersion>,

    #[serde(default)]
    authority: Option<String>,

    #[serde(default)]
    private_key_hash: Option<String>,
}

impl From<SigningSectionFile> for SigningConfig {
    fn from(file: SigningSectionFile) -> Self {
        Self {
            version: file.version.unwrap_or_default(),
            authority: file.authority,
            private_key_hash: file.private_key_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
version = 2
authority = "test"
private_key_hash = "abcd1234"
"#;
        let file: SigningSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = SigningConfig::from(file);
        assert_eq!(config.version(), BISignVersion::V2);
        assert_eq!(config.authority(), Some("test"));
        assert_eq!(config.private_key_hash(), Some("abcd1234"));
    }

    #[test]
    fn default() {
        let toml = "";
        let file: SigningSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = SigningConfig::from(file);
        assert_eq!(config.version(), BISignVersion::V3);
        assert!(config.authority().is_none());
    }
}
