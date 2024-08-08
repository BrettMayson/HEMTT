use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
/// Files config
pub struct FilesConfig {
    /// Files to exclude from the PBO
    exclude: Vec<String>,
}

impl FilesConfig {
    /// Files to exclude from the PBO
    pub const fn exclude(&self) -> &Vec<String> {
        &self.exclude
    }

    /// Files to exclude from the PBO
    pub(crate) fn exclude_mut(&mut self) -> &mut Vec<String> {
        &mut self.exclude
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// Section of the project.toml file for files
pub struct FilesSectionFile {
    #[serde(default)]
    /// Files to exclude from the PBO
    pub exclude: Vec<String>,
}

impl From<FilesSectionFile> for FilesConfig {
    fn from(file: FilesSectionFile) -> Self {
        Self {
            exclude: file.exclude,
        }
    }
}
