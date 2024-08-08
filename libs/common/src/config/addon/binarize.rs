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
