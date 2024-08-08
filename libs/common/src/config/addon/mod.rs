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
        AddonFile::from_file(path).map(Into::into)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Configuration file for an addon
pub struct AddonFile {
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

impl AddonFile {
    pub fn from_file(path: &std::path::Path) -> Result<Self, Error> {
        let file = std::fs::read_to_string(path)?;

        let config: Self = toml::from_str(&file)?;

        let see_more =
            "See <https://brettmayson.github.io/HEMTT/configuration/addon> for more information.";

        if file.contains("preprocess = ") {
            return Err(Error::ConfigInvalid(format!("`preprocess = {{}}` is deprecated, use `[rapify] enabled = false` instead. {see_more}")));
        }

        DEPRECATION.call_once(|| {
            if file.contains("[preprocess]") {
                deprecated(path, "[preprocess]", "[rapify]", Some(see_more));
            }

            if file.contains("no_bin") {
                deprecated(path, "no_bin", "binarize.exclude", Some(see_more));
            }

            if file.contains("preprocess") {
                deprecated(path, "preprocess", "rapify", Some(see_more));
            }

            if !config.exclude.is_empty() {
                deprecated(path, "exclude", "files.exclude", Some(see_more));
            }
        });

        Ok(config)
    }
}

impl From<AddonFile> for AddonConfig {
    fn from(file: AddonFile) -> Self {
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
