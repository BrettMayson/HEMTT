mod binarize;
mod files;
mod rapify;

use std::{collections::HashMap, sync::Once};

use serde::{Deserialize, Serialize};

use crate::Error;

use super::deprecated;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct AddonConfig {
    /// Rapifier config
    rapify: rapify::RapifyConfig,

    /// Binarze config
    binarize: binarize::BinarizeConfig,

    /// Properties to add to the pbo
    properties: HashMap<String, String>,

    /// Files to exclude from the pbo
    files: files::FilesConfig,
}

impl AddonConfig {
    #[must_use]
    /// Rapify config
    pub const fn rapify(&self) -> &rapify::RapifyConfig {
        &self.rapify
    }

    #[must_use]
    /// Binirize config
    pub const fn binarize(&self) -> &binarize::BinarizeConfig {
        &self.binarize
    }

    #[must_use]
    /// Properties to add to the pbo
    pub const fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[must_use]
    /// Files to exclude from the pbo
    pub const fn files(&self) -> &files::FilesConfig {
        &self.files
    }

    /// Load a configuration from a file.
    ///
    /// # Errors
    /// [`crate::error::Error::Io`] if the file cannot be read
    /// [`crate::error::Error::Toml`] if the file is not valid toml
    pub fn from_file(path: &std::path::Path) -> Result<Self, crate::error::Error> {
        AddonConfigFile::from_file(path).map(Into::into)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration file for an addon
pub struct AddonConfigFile {
    #[serde(default)]
    #[serde(alias = "preprocess")]
    rapify: rapify::RapifySectionFile,

    #[serde(default)]
    no_bin: Vec<String>,

    #[serde(default)]
    binarize: binarize::BinarizeSectionFile,

    #[serde(default)]
    properties: HashMap<String, String>,

    #[serde(default)]
    exclude: Vec<String>,

    #[serde(default)]
    files: files::FilesSectionFile,
}

static DEPRECATION: Once = Once::new();

impl AddonConfigFile {
    pub fn from_file(path: &std::path::Path) -> Result<Self, Error> {
        Self::from_str(&fs_err::read_to_string(path)?, &path.display().to_string())
    }

    pub fn from_str(content: &str, path: &str) -> Result<Self, Error> {
        let config: Self = toml::from_str(content)?;

        let see_more = "See <https://hemtt.dev/configuration/addon> for more information.";

        if content.contains("preprocess = ") || content.contains("preprocess=") {
            return Err(Error::ConfigInvalid(format!(
                "`preprocess = {{}}` is deprecated, use `[rapify] enabled = false` instead. {see_more}"
            )));
        }

        DEPRECATION.call_once(|| {
            if content.contains("[preprocess]") {
                deprecated(path, "[preprocess]", "[rapify]", Some(see_more));
            }

            if content.contains("no_bin") {
                deprecated(path, "no_bin", "binarize.exclude", Some(see_more));
            }

            if !config.exclude.is_empty() {
                deprecated(path, "exclude", "files.exclude", Some(see_more));
            }
        });

        Ok(config)
    }
}

impl From<AddonConfigFile> for AddonConfig {
    fn from(file: AddonConfigFile) -> Self {
        Self {
            rapify: file.rapify.into(),
            binarize: {
                let mut binarize: binarize::BinarizeConfig = file.binarize.into();
                binarize.exclude_mut().extend(file.no_bin);
                binarize
            },
            properties: file.properties,
            files: {
                let mut files: files::FilesConfig = file.files.into();
                files.exclude_mut().extend(file.exclude);
                files
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_defined() {
        let toml = r#"
[rapify]
enabled = true

[binarize]
enabled = true

[properties]
test = "test"

[files]
exclude = ["test"]

"#;
        let file: AddonConfigFile = toml::from_str(toml).expect("failed to deserialize");
        let config = AddonConfig::from(file);
        assert!(config.rapify().enabled());
        assert!(config.binarize().enabled());
        assert_eq!(config.properties().get("test"), Some(&"test".to_string()));
        assert_eq!(config.files().exclude(), &["test"]);
    }

    #[test]
    fn default() {
        let toml = "";
        let file: AddonConfigFile = toml::from_str(toml).expect("failed to deserialize");
        let config = AddonConfig::from(file);
        assert!(config.rapify().enabled());
        assert!(config.binarize().enabled());
        assert!(config.properties().is_empty());
        assert!(config.files().exclude().is_empty());
    }

    #[test]
    #[tracing_test::traced_test]
    fn deprecated() {
        let toml = r#"
no_bin = ["test"]
exclude = ["test"]

[binarize]
enabled = true

[properties]
test = "test"

[preprocess]
enabled = true
"#;
        let file = AddonConfigFile::from_str(toml, "test").expect("failed to deserialize");
        let config = AddonConfig::from(file);
        assert!(config.rapify().enabled());
        assert!(config.binarize().enabled());
        assert_eq!(config.properties().get("test"), Some(&"test".to_string()));
        assert_eq!(config.files().exclude(), &["test"]);

        assert!(logs_contain("Use of deprecated key '[preprocess]'"));
        assert!(logs_contain("Use of deprecated key 'no_bin'"));
        assert!(logs_contain("Use of deprecated key 'exclude'"));
    }

    #[test]
    fn deprecated_preprocess() {
        let toml = "
preprocess = true
";
        assert!(AddonConfigFile::from_str(toml, "test").is_err());
    }
}
