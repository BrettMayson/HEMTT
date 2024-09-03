use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Binarize config
pub struct BinarizeConfig {
    /// Is binarize enabled
    enabled: bool,
    /// Files to exclude from binarize
    exclude: Vec<String>,
}

impl BinarizeConfig {
    /// Is binarize enabled
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Files to exclude from binarize
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }

    pub(crate) fn exclude_mut(&mut self) -> &mut Vec<String> {
        &mut self.exclude
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Section of the project.toml file for binarize
pub struct BinarizeSectionFile {
    #[serde(default)]
    /// Is binarize enabled
    pub enabled: Option<bool>,
    #[serde(default)]
    /// Files to exclude from binarize
    pub exclude: Vec<String>,
}

impl From<BinarizeSectionFile> for BinarizeConfig {
    fn from(file: BinarizeSectionFile) -> Self {
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
        let file: BinarizeSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BinarizeConfig::from(file);
        assert!(!config.enabled());
        assert_eq!(config.exclude(), &["test".to_string()]);
    }

    #[test]
    fn exlucde_defined() {
        let toml = r#"exclude = ["test"]"#;
        let file: BinarizeSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BinarizeConfig::from(file);
        assert!(config.enabled());
        assert_eq!(config.exclude(), &["test".to_string()]);
    }

    #[test]
    fn enabled_defined_false() {
        let toml = r"enabled = false";
        let file: BinarizeSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BinarizeConfig::from(file);
        assert!(!config.enabled());
        assert!(config.exclude().is_empty());
    }

    #[test]
    fn enabled_defined_true() {
        let toml = r"enabled = true";
        let file: BinarizeSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BinarizeConfig::from(file);
        assert!(config.enabled());
        assert!(config.exclude().is_empty());
    }

    #[test]
    fn empty() {
        let toml = "";
        let file: BinarizeSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = BinarizeConfig::from(file);
        assert!(config.enabled());
        assert!(config.exclude().is_empty());
    }
}
