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
