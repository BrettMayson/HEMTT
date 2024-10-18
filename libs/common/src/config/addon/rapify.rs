use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Rapify config
pub struct RapifyConfig {
    enabled: bool,
    exclude: Vec<String>,
}

impl RapifyConfig {
    #[must_use]
    /// Is rapify enabled
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    /// Files to exclude from rapify
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Section of the project.toml file for rapify
pub struct RapifySectionFile {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    exclude: Vec<String>,
}

impl From<RapifySectionFile> for RapifyConfig {
    fn from(file: RapifySectionFile) -> Self {
        Self {
            enabled: file.enabled.unwrap_or(true),
            exclude: file.exclude,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
enabled = false
exclude = ["test"]
"#;
        let file: RapifySectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = RapifyConfig::from(file);
        assert!(!config.enabled());
        assert_eq!(config.exclude(), &["test"]);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: RapifySectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = RapifyConfig::from(file);
        assert!(config.enabled());
        assert!(config.exclude().is_empty());
    }
}
