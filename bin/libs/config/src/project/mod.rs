use std::{collections::HashMap, path::Path, str::FromStr};

use hemtt_bin_error::Error;
use serde::{Deserialize, Serialize};

mod asc;
mod files;
mod hemtt;
mod signing;
mod version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// The name of the project
    name: String,
    /// Prefix for the project
    prefix: String,
    /// Main prefix for the project
    mainprefix: Option<String>,

    #[serde(default)]
    /// version of the project
    version: version::Options,

    #[serde(default)]
    /// Properties to be added to built PBOs
    properties: HashMap<String, String>,

    #[serde(default)]
    /// Files to be included in the root of the project, supports glob patterns
    files: files::Options,

    #[serde(default)]
    hemtt: hemtt::Features,

    #[serde(default)]
    signing: signing::Options,

    #[serde(default)]
    asc: asc::Options,
}

impl Configuration {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn version(&self) -> &version::Options {
        &self.version
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub const fn mainprefix(&self) -> Option<&String> {
        self.mainprefix.as_ref()
    }

    #[must_use]
    /// Properties to be added to built PBOs
    pub const fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }

    #[must_use]
    /// Files to be included / excluded in the root of the project, supports glob patterns
    pub const fn files(&self) -> &files::Options {
        &self.files
    }

    #[must_use]
    pub const fn hemtt(&self) -> &hemtt::Features {
        &self.hemtt
    }

    #[must_use]
    pub const fn signing(&self) -> &signing::Options {
        &self.signing
    }

    #[must_use]
    pub const fn asc(&self) -> &asc::Options {
        &self.asc
    }

    /// Load a configuration from a file.
    ///
    /// # Errors
    ///
    /// If the file cannot be read, or if the file is not valid TOML, or if the
    /// file does not contain a valid configuration, an error is returned.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file = std::fs::read_to_string(path)?;
        let config = Self::from_str(&file)?;

        // Validate
        if config.prefix.is_empty() {
            return Err(Error::ConfigInvalid("prefix cannot be empty".to_string()));
        }

        Ok(config)
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::from)
    }
}
