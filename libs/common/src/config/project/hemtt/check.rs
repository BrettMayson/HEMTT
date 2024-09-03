use serde::{Deserialize, Serialize};

use crate::config::pdrive::PDriveOption;

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone)]
/// Configuration for `hemtt check`
pub struct CheckOptions {
    pdrive: PDriveOption,
}

impl CheckOptions {
    /// Can HEMTT look in the P drive for includes?
    pub const fn pdrive(&self) -> &PDriveOption {
        &self.pdrive
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Dev specific configuration
pub struct CheckOptionsFile {
    #[serde(default)]
    pdrive: Option<PDriveOption>,
}

impl From<CheckOptionsFile> for CheckOptions {
    fn from(file: CheckOptionsFile) -> Self {
        Self {
            pdrive: file.pdrive.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
pdrive = "disallow"
"#;
        let file: CheckOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = CheckOptions::from(file);
        assert_eq!(config.pdrive(), &PDriveOption::Disallow);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: CheckOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = CheckOptions::from(file);
        assert_eq!(config.pdrive(), &PDriveOption::Ignore);
    }
}
