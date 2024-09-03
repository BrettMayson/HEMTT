use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for `hemtt dev`
pub struct DevOptions {
    exclude: Vec<String>,
}

impl DevOptions {
    /// Files to exclude from the PBO
    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
/// Dev specific configuration
pub struct DevOptionsFile {
    #[serde(default)]
    exclude: Vec<String>,
}

impl From<DevOptionsFile> for DevOptions {
    fn from(file: DevOptionsFile) -> Self {
        Self {
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
exclude = ["test"]
"#;
        let file: DevOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = DevOptions::from(file);
        assert_eq!(config.exclude(), &["test"]);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: DevOptionsFile = toml::from_str(toml).expect("failed to deserialize");
        let config = DevOptions::from(file);
        assert!(config.exclude().is_empty());
    }
}
