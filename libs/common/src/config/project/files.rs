use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Files config
pub struct FilesConfig {
    /// Files to include in the output folder, supports glob patterns
    include: Vec<String>,
    /// Files to exclude from the PBO
    exclude: Vec<String>,
}

impl FilesConfig {
    /// Files to include in the output folder, supports glob patterns
    pub fn include(&self) -> &[String] {
        &self.include
    }

    /// Files to exclude from the PBO
    pub fn exclude(&self) -> &[String] {
        &self.exclude
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilesSectionFile {
    #[serde(default)]
    /// Files to be included in the output folder, supports glob patterns
    include: Vec<String>,
    #[serde(default)]
    /// Files to be excluded from being included in PBO files, supports glob patterns
    exclude: Vec<String>,
}

impl From<FilesSectionFile> for FilesConfig {
    fn from(file: FilesSectionFile) -> Self {
        Self {
            include: {
                let mut files = file
                    .include
                    .iter()
                    .map(|i| {
                        if i.starts_with('/') {
                            i.to_string()
                        } else {
                            format!("/{i}")
                        }
                    })
                    .collect::<Vec<_>>();
                for default in [
                    "/mod.cpp",
                    "/meta.cpp",
                    "/LICENSE",
                    "/logo_ca.paa",
                    "/logo_co.paa",
                ]
                .iter()
                .map(std::string::ToString::to_string)
                {
                    files.push(default.clone());
                }
                files.sort();
                files.dedup();
                files
            },
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
include = ["test"]
exclude = ["test"]
"#;
        let file: FilesSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = FilesConfig::from(file);
        assert!(config.include().contains(&"/test".to_string()));
        assert_eq!(config.exclude(), &["test"]);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: FilesSectionFile = toml::from_str(toml).expect("failed to deserialize");
        let config = FilesConfig::from(file);
        assert!(config.include().contains(&"/mod.cpp".to_string()));
        assert!(config.exclude().is_empty());
    }
}
