use serde::{Deserialize, Serialize};

use crate::config::pdrive::PDriveOption;

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Clone)]
/// Configuration for `hemtt build`
pub struct BuildOptions {
    optional_mod_folders: bool,
    pdrive: PDriveOption,
}

impl BuildOptions {
    /// Should optionals be built into their own mod?
    pub const fn optional_mod_folders(&self) -> bool {
        self.optional_mod_folders
    }

    /// Can HEMTT look in the P drive for includes?
    pub const fn pdrive(&self) -> &PDriveOption {
        &self.pdrive
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Build specific configuration
pub struct BuildOptionsFile {
    #[serde(default)]
    optional_mod_folders: Option<bool>,
    #[serde(default)]
    pdrive: Option<PDriveOption>,
}

impl From<BuildOptionsFile> for BuildOptions {
    fn from(file: BuildOptionsFile) -> Self {
        Self {
            optional_mod_folders: file.optional_mod_folders.unwrap_or(true),
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
optional_mod_folders = false
pdrive = "disallow"
"#;
        let file: BuildOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BuildOptions::from(file);
        assert!(!config.optional_mod_folders());
        assert_eq!(config.pdrive(), &PDriveOption::Disallow);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: BuildOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BuildOptions::from(file);
        assert!(config.optional_mod_folders());
        assert_eq!(config.pdrive(), &PDriveOption::Ignore);
    }
}
